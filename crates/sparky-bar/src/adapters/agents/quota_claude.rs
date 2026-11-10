use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use chrono::DateTime;
use serde_json::Value;
use zeroize::Zeroizing;

use crate::domain::agents::{Period, Quota};

const ENDPOINT: &str = "https://api.anthropic.com/api/oauth/usage";

pub fn read(config: &Path) -> Vec<Quota> {
    let Ok(bytes) = fs::read(config.join(".credentials.json")) else {
        return Vec::new();
    };
    let bytes = Zeroizing::new(bytes);
    let Ok(credentials) = serde_json::from_slice::<Value>(&bytes) else {
        return Vec::new();
    };
    let oauth = &credentials["claudeAiOauth"];
    let Some(token) = oauth["accessToken"]
        .as_str()
        .filter(|token| safe_token(token))
    else {
        return Vec::new();
    };
    let expires_at = oauth["expiresAt"].as_i64().unwrap_or(0);
    if expires_at <= chrono::Utc::now().timestamp_millis() {
        return Vec::new();
    }
    fetch(token)
        .and_then(|body| serde_json::from_slice::<Value>(&body).ok())
        .map_or_else(Vec::new, |body| parse(&body))
}

fn safe_token(token: &str) -> bool {
    !token.is_empty()
        && token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn fetch(token: &str) -> Option<Vec<u8>> {
    let mut child = Command::new("curl")
        .args([
            "--silent",
            "--show-error",
            "--fail",
            "--max-time",
            "6",
            "--proto",
            "=https",
            "--config",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let config = format!(
        "url = \"{ENDPOINT}\"\nheader = \"Authorization: Bearer {token}\"\nheader = \"anthropic-beta: oauth-2025-04-20\"\nheader = \"Accept: application/json\"\n"
    );
    let written = child
        .stdin
        .take()
        .is_some_and(|mut stdin| stdin.write_all(config.as_bytes()).is_ok());
    if !written {
        let _ = child.kill();
        let _ = child.wait();
        return None;
    }
    let output = child.wait_with_output().ok()?;
    output.status.success().then_some(output.stdout)
}

fn parse(body: &Value) -> Vec<Quota> {
    [
        ("five_hour", Period::FiveHours),
        ("seven_day", Period::Weekly),
    ]
    .into_iter()
    .filter_map(|(key, period)| {
        let percent = body[key]["utilization"].as_f64()?;
        if !percent.is_finite() {
            return None;
        }
        let resets_at = body[key]["resets_at"]
            .as_str()
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|date| date.timestamp());
        Some(Quota {
            period,
            used_percent: percent.clamp(0.0, 100.0).round() as u8,
            resets_at,
        })
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::{parse, safe_token};
    use crate::domain::agents::Period;

    #[test]
    fn rejeita_token_com_injecao_de_configuracao() {
        assert!(safe_token("sk-ant-oat01_abc.def"));
        assert!(!safe_token("abc\nheader = \"X: y\""));
    }

    #[test]
    fn converte_limites_sem_inventar_janelas_ausentes() {
        let body = serde_json::json!({"five_hour":{"utilization":31.4,"resets_at":"2026-11-10T21:00:00Z"},"seven_day":{"utilization":105.0}});
        let quotas = parse(&body);
        assert_eq!(quotas.len(), 2);
        assert_eq!(quotas[0].period, Period::FiveHours);
        assert_eq!(quotas[0].used_percent, 31);
        assert_eq!(quotas[1].used_percent, 100);
    }
}
