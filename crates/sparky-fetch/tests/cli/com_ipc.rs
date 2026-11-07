use std::time::{Duration, Instant};

use super::compositor::{Kind, serve};
use super::run::{entry, fetch, json};

const CAMPOS: &str = "compositor,session,monitors,resolution,theme,colors";

fn com(kind: Kind, argumentos: &[&str]) -> (super::run::Run, Duration) {
    let home = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let socket = serve(home.path(), kind);
    let inicio = Instant::now();
    let run = fetch(argumentos, home.path(), Some(&socket), "pt_BR.UTF-8");
    (run, inicio.elapsed())
}

#[test]
fn com_o_compositor_os_campos_do_sparky_ficam_prontos() {
    let (run, _) = com(
        Kind::Current,
        &["--json", "--no-config", "--fields", CAMPOS],
    );
    let relatorio = json(&run);
    assert_eq!(entry(&relatorio, "compositor")["value"], "Sparky 0.30.0");
    assert_eq!(entry(&relatorio, "session")["value"], "Wayland · wayland-1");
    assert_eq!(entry(&relatorio, "theme")["value"], "tokyo-night");
    assert_eq!(
        entry(&relatorio, "monitors")["value"][0],
        "DP-1 · LG ULTRAGEAR"
    );
    assert_eq!(
        entry(&relatorio, "resolution")["value"][0],
        "2560x1440 @ 144 Hz (×1.25)"
    );
    assert_eq!(entry(&relatorio, "colors")["value"][0], "#7aa2f7");
}

#[test]
fn compositor_antigo_recusa_info_e_mantem_os_monitores() {
    let (run, _) = com(Kind::Old, &["--json", "--no-config", "--fields", CAMPOS]);
    let relatorio = json(&run);
    assert_eq!(entry(&relatorio, "compositor")["reason"], "unsupported");
    assert_eq!(entry(&relatorio, "colors")["reason"], "unsupported");
    assert_eq!(entry(&relatorio, "monitors")["status"], "ready");
}

#[test]
fn compositor_mudo_estoura_o_prazo_sem_travar() {
    let argumentos = [
        "--json",
        "--no-config",
        "--timeout",
        "200",
        "--fields",
        CAMPOS,
    ];
    let (run, tempo) = com(Kind::Silent, &argumentos);
    assert!(tempo < Duration::from_secs(2), "{tempo:?}");
    assert_eq!(entry(&json(&run), "compositor")["status"], "timed_out");
}

#[test]
fn snapshot_do_texto_simples_com_compositor() {
    let (run, _) = com(
        Kind::Current,
        &["--plain", "--no-config", "--fields", CAMPOS],
    );
    assert_eq!(run.stdout, include_str!("../snapshots/plain_ipc.txt"));
}

#[test]
fn snapshot_do_visual_com_compositor() {
    let argumentos = [
        "--format",
        "visual",
        "--color",
        "never",
        "--width",
        "80",
        "--no-config",
        "--fields",
        CAMPOS,
    ];
    let (run, _) = com(Kind::Current, &argumentos);
    assert_eq!(run.stdout, include_str!("../snapshots/visual_ipc.txt"));
}
