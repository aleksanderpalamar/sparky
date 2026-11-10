use std::io::Write;

use super::{FileState, Usage, claude_line, codex_line, read};

#[test]
fn claude_conta_mensagem_uma_vez_incluindo_cache() {
    let day = chrono::Local::now().date_naive();
    let stamp = chrono::Local::now().to_rfc3339();
    let value = serde_json::json!({"type":"assistant","timestamp":stamp,"message":{"id":"m1","usage":{"input_tokens":2,"cache_creation_input_tokens":3,"cache_read_input_tokens":4,"output_tokens":5}}});
    let mut state = FileState::default();
    claude_line(&value, day, &mut state);
    claude_line(&value, day, &mut state);
    assert_eq!(state.tokens, 14);
    let mut expanded = value;
    expanded["message"]["usage"]["output_tokens"] = serde_json::json!(7);
    claude_line(&expanded, day, &mut state);
    assert_eq!(state.tokens, 16);
}

#[test]
fn codex_conta_deltas_sem_duplicar_ultima_amostra() {
    let day = chrono::Local::now().date_naive();
    let stamp = chrono::Local::now().to_rfc3339();
    let mut state = FileState::default();
    for total in [10, 20, 20] {
        let value = serde_json::json!({"type":"event_msg","timestamp":stamp,"payload":{"type":"token_count","info":{"total_token_usage":{"total_tokens":total}}}});
        codex_line(&value, day, &mut state);
    }
    assert_eq!(state.tokens, 20);
}

#[test]
fn linha_parcial_aguarda_proxima_leitura() {
    let file = tempfile::NamedTempFile::new().unwrap();
    let stamp = chrono::Local::now().to_rfc3339();
    let value = serde_json::json!({"type":"assistant","timestamp":stamp,"message":{"id":"m1","usage":{"input_tokens":2}}});
    let line = serde_json::to_string(&value).unwrap();
    std::fs::write(file.path(), &line).unwrap();
    let mut state = FileState::default();
    let day = chrono::Local::now().date_naive();
    read(file.path(), &mut state, day, "\"assistant\"", claude_line);
    assert_eq!(state.tokens, 0);
    let mut handle = std::fs::OpenOptions::new()
        .append(true)
        .open(file.path())
        .unwrap();
    handle.write_all(b"\n").unwrap();
    read(file.path(), &mut state, day, "\"assistant\"", claude_line);
    assert_eq!(state.tokens, 2);
}

#[test]
fn mensagem_repetida_em_arquivos_distintos_conta_uma_vez() {
    let claude = tempfile::tempdir().unwrap();
    let codex = tempfile::tempdir().unwrap();
    let stamp = chrono::Local::now().to_rfc3339();
    let value = serde_json::json!({"type":"assistant","timestamp":stamp,"message":{"id":"m1","usage":{"input_tokens":4,"output_tokens":2}}});
    let line = format!("{value}\n");
    std::fs::write(claude.path().join("one.jsonl"), &line).unwrap();
    std::fs::write(claude.path().join("two.jsonl"), &line).unwrap();
    let mut usage = Usage::default();
    assert_eq!(usage.today(claude.path(), codex.path()), (6, 0));
    assert_eq!(usage.today(claude.path(), codex.path()), (6, 0));
}
