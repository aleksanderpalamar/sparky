use super::protection::guarded;
use crate::context::Process;

fn process(pid: u32, name: &str) -> Process {
    Process {
        pid,
        parent: 1,
        name: name.to_owned(),
    }
}

#[test]
fn mostra_gravacao_chamada_escolhidos_e_janela_em_foco() {
    let processes = [
        process(10, "obs"),
        process(11, "discord"),
        process(12, "rustc"),
        process(13, "blender"),
        process(14, "obs"),
    ];
    let names = guarded(
        &processes,
        &["blender".to_owned()],
        &["steam_app_1091500".to_owned()],
    );
    assert_eq!(names, ["blender", "discord", "obs", "steam_app_1091500"]);
}

#[test]
fn a_lista_mostrada_tem_limite() {
    let processes: Vec<Process> = (0..40).map(|pid| process(pid, "obs")).collect();
    let windows: Vec<String> = (0..40).map(|index| format!("app-{index:02}")).collect();
    assert!(guarded(&processes, &[], &windows).len() <= 8);
}
