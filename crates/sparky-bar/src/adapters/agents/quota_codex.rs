use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::domain::agents::{Period, Quota};

pub fn read(binary: &Path) -> Vec<Quota> {
    if env::var_os("OPENAI_API_KEY").is_some() || api_key_mode() {
        return Vec::new();
    }
    let Ok(mut child) = Command::new(binary)
        .args(["-s", "read-only", "-a", "on-request", "app-server"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return Vec::new();
    };
    let result = probe(&mut child);
    let _ = child.kill();
    let _ = child.wait();
    result.unwrap_or_default()
}

fn api_key_mode() -> bool {
    let home = env::var_os("CODEX_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".codex")));
    let Some(home) = home else {
        return false;
    };
    let Ok(bytes) = fs::read(home.join("auth.json")) else {
        return false;
    };
    serde_json::from_slice::<Value>(&bytes)
        .ok()
        .is_some_and(|auth| auth["auth_mode"] == "apikey")
}

fn probe(child: &mut Child) -> Option<Vec<Quota>> {
    let mut input = child.stdin.take()?;
    let output = child.stdout.take()?;
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(output).lines() {
            let Ok(line) = line else { break };
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    send(
        &mut input,
        json!({"id":1,"method":"initialize","params":{"clientInfo":{"name":"sparky-agents","version":"1"}}}),
    )?;
    response(&receiver, 1, Duration::from_secs(8))?;
    send(&mut input, json!({"method":"initialized","params":{}}))?;
    send(
        &mut input,
        json!({"id":2,"method":"account/rateLimits/read","params":{}}),
    )?;
    let answer = response(&receiver, 2, Duration::from_secs(8))?;
    Some(parse(&answer))
}

fn send(input: &mut ChildStdin, value: Value) -> Option<()> {
    serde_json::to_writer(&mut *input, &value).ok()?;
    input.write_all(b"\n").ok()?;
    input.flush().ok()
}

fn response(receiver: &Receiver<String>, id: u64, timeout: Duration) -> Option<Value> {
    let deadline = Instant::now() + timeout;
    loop {
        let wait = deadline.checked_duration_since(Instant::now())?;
        let line = receiver.recv_timeout(wait).ok()?;
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if value["id"].as_u64() == Some(id) {
            return Some(value);
        }
    }
}

fn parse(answer: &Value) -> Vec<Quota> {
    let result = &answer["result"];
    let limits = if result["rateLimits"]["primary"].is_object() {
        &result["rateLimits"]
    } else {
        let Some(items) = result["rateLimitsByLimitId"].as_object() else {
            return Vec::new();
        };
        if items.len() != 1 {
            return Vec::new();
        }
        let Some(limits) = items.values().next() else {
            return Vec::new();
        };
        limits
    };
    [
        (&limits["primary"], Period::FiveHours),
        (&limits["secondary"], Period::Weekly),
    ]
    .into_iter()
    .filter_map(|(window, period)| {
        let percent = window["usedPercent"].as_f64()?;
        if !percent.is_finite() {
            return None;
        }
        Some(Quota {
            period,
            used_percent: percent.clamp(0.0, 100.0).round() as u8,
            resets_at: window["resetsAt"].as_i64(),
        })
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::domain::agents::Period;

    #[test]
    fn extrai_janelas_da_conta_sem_inventar_limte_ausente() {
        let answer = serde_json::json!({"result":{"rateLimitsByLimitId":{"default":{"primary":{"usedPercent":24.8,"resetsAt":1794348000},"secondary":{"usedPercent":42.0,"resetsAt":1795000000}}}}});
        let quotas = parse(&answer);
        assert_eq!(quotas.len(), 2);
        assert_eq!(quotas[0].period, Period::FiveHours);
        assert_eq!(quotas[0].used_percent, 25);
        assert_eq!(quotas[1].period, Period::Weekly);
    }

    #[test]
    fn prefere_limite_global_e_nao_escolhe_modelo_ambiguo() {
        let answer = serde_json::json!({"result":{"rateLimits":{"primary":{"usedPercent":30.0}},"rateLimitsByLimitId":{"a":{"primary":{"usedPercent":10.0}},"b":{"primary":{"usedPercent":80.0}}}}});
        assert_eq!(parse(&answer)[0].used_percent, 30);
        let answer = serde_json::json!({"result":{"rateLimitsByLimitId":{"a":{"primary":{"usedPercent":10.0}},"b":{"primary":{"usedPercent":80.0}}}}});
        assert!(parse(&answer).is_empty());
    }
}
