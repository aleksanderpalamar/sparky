use std::time::{Duration, Instant};

use super::run::{entry, fetch, json};

const SPARKY: [&str; 6] = [
    "compositor",
    "session",
    "monitors",
    "resolution",
    "theme",
    "colors",
];

fn pasta() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn sem_ipc_o_linux_continua_e_o_sparky_diz_por_que_falta() {
    let home = pasta();
    let run = fetch(&["--json", "--no-config"], home.path(), None, "pt_BR.UTF-8");
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    let relatorio = json(&run);
    assert_eq!(relatorio["entries"].as_array().map(Vec::len), Some(18));
    assert_eq!(entry(&relatorio, "kernel")["status"], "ready");
    for campo in SPARKY {
        assert_eq!(entry(&relatorio, campo)["status"], "unavailable", "{campo}");
        assert_eq!(
            entry(&relatorio, campo)["reason"],
            "no_compositor",
            "{campo}"
        );
    }
}

#[test]
fn no_pipe_sai_texto_simples_sem_escapes() {
    let home = pasta();
    let run = fetch(&["--no-config"], home.path(), None, "pt_BR.UTF-8");
    assert_eq!(run.code, Some(0));
    assert!(!run.stdout.contains('\x1b'));
    assert!(run.stdout.lines().any(|line| line.starts_with("Kernel: ")));
    assert!(run.stdout.contains("Compositor: N/A"));
}

#[test]
fn visual_com_cor_forcada_usa_ansi_de_24_bits() {
    let home = pasta();
    let argumentos = [
        "--format",
        "visual",
        "--color",
        "always",
        "--width",
        "80",
        "--no-config",
    ];
    let run = fetch(&argumentos, home.path(), None, "pt_BR.UTF-8");
    assert!(run.stdout.contains("\x1b[38;2;"));
    assert!(run.stdout.lines().count() >= 11);
}

#[test]
fn os_rotulos_seguem_o_idioma_da_sessao() {
    let home = pasta();
    let ingles = fetch(
        &["--plain", "--fields", "os,memory", "--no-config"],
        home.path(),
        None,
        "en_US.UTF-8",
    );
    let portugues = fetch(
        &["--plain", "--fields", "os,memory", "--no-config"],
        home.path(),
        None,
        "pt_BR.UTF-8",
    );
    assert!(ingles.stdout.starts_with("OS: "), "{}", ingles.stdout);
    assert!(ingles.stdout.contains("\nMemory: "));
    assert!(
        portugues.stdout.starts_with("Sistema: "),
        "{}",
        portugues.stdout
    );
}

#[test]
fn termina_rapido_mesmo_sem_compositor() {
    let home = pasta();
    let inicio = Instant::now();
    let run = fetch(
        &["--plain", "--no-config"],
        home.path(),
        None,
        "pt_BR.UTF-8",
    );
    assert_eq!(run.code, Some(0));
    assert!(inicio.elapsed() < Duration::from_secs(2));
}
