use super::recognizer::recognize;
use super::scene::{Load, Process, Scene, Window};
use crate::domain::{Activities, Activity};

fn process(pid: u32, parent: u32, name: &str) -> Process {
    Process {
        pid,
        parent,
        name: name.to_owned(),
    }
}

fn desktop() -> Vec<Process> {
    vec![
        process(1, 0, "systemd"),
        process(1233, 1, "sparky"),
        process(2268, 1233, "steam"),
    ]
}

fn with(mut processes: Vec<Process>, extra: &[Process]) -> Scene {
    processes.extend_from_slice(extra);
    Scene {
        processes: Some(processes),
        windows: Some(Vec::new()),
        load: Load {
            cpu_busy_percent: Some(40.0),
            ..Load::default()
        },
    }
}

fn activities(list: &[Activity]) -> Activities {
    list.iter().copied().collect()
}

#[test]
fn jogo_da_steam_e_reconhecido_pela_arvore_e_nao_pelo_nome() {
    let scene = with(
        desktop(),
        &[
            process(3000, 2268, "reaper"),
            process(3001, 3000, "wine64-preload"),
            process(3002, 3001, "Cyberpunk2077.e"),
        ],
    );
    assert!(recognize(&scene).contains(Activity::Gaming));
}

#[test]
fn steam_aberta_sem_jogo_nao_e_jogo() {
    let scene = with(desktop(), &[process(3000, 2268, "reaper")]);
    assert!(!recognize(&scene).contains(Activity::Gaming));
}

#[test]
fn jogo_com_obs_e_discord_sao_atividades_simultaneas() {
    let scene = with(
        desktop(),
        &[
            process(3000, 2268, "reaper"),
            process(3002, 3000, "ets2"),
            process(4000, 1233, "obs"),
            process(5000, 1233, "Discord"),
        ],
    );
    assert_eq!(
        recognize(&scene),
        activities(&[
            Activity::Gaming,
            Activity::Recording,
            Activity::Communication
        ])
    );
}

#[test]
fn compilacao_e_editor_indicam_desenvolvimento() {
    let scene = with(
        desktop(),
        &[
            process(6000, 1233, "rust-analyzer"),
            process(6001, 1233, "rustc"),
        ],
    );
    assert_eq!(
        recognize(&scene),
        activities(&[Activity::Development, Activity::Compiling])
    );
}

#[test]
fn memoria_quase_esgotada_ou_travando_e_pressao_critica() {
    let mut scene = with(desktop(), &[]);
    scene.load.memory_available_percent = Some(3.0);
    assert!(recognize(&scene).contains(Activity::CriticalPressure));
    scene.load = Load {
        memory_stall_avg10: Some(12.5),
        ..scene.load
    };
    scene.load.memory_available_percent = Some(50.0);
    assert!(recognize(&scene).contains(Activity::CriticalPressure));
}

#[test]
fn sem_atividade_e_com_cpu_baixa_e_ocioso() {
    let mut scene = with(desktop(), &[]);
    scene.load.cpu_busy_percent = Some(2.0);
    assert_eq!(recognize(&scene), activities(&[Activity::Idle]));
    scene.load.cpu_busy_percent = Some(30.0);
    scene.windows = Some(vec![Window {
        app_id: "firefox".to_owned(),
        pid: Some(9),
        focused: true,
        fullscreen: false,
    }]);
    assert_eq!(recognize(&scene), activities(&[Activity::Desktop]));
}

#[test]
fn sem_nenhum_dado_o_contexto_e_desconhecido() {
    assert_eq!(
        recognize(&Scene::default()),
        activities(&[Activity::Unknown])
    );
}

#[test]
fn ciclo_na_arvore_nao_trava_o_reconhecimento() {
    let scene = with(Vec::new(), &[process(10, 11, "a"), process(11, 10, "b")]);
    assert!(!recognize(&scene).contains(Activity::Gaming));
}
