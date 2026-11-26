use super::planner::{Contention, Purpose, plan};
use super::protection::protected;
use crate::context::Process;
use crate::domain::{Activities, Activity, Change, Identity, IoClass};
use crate::telemetry::consumers::{Consumer, Usage};

fn process(pid: u32, parent: u32, name: &str) -> Process {
    Process {
        pid,
        parent,
        name: name.to_owned(),
    }
}

fn consumer(pid: u32, name: &str) -> Consumer {
    Consumer {
        identity: Identity { pid, started: 1 },
        parent: 1,
        name: name.to_owned(),
        ticks: 0,
    }
}

fn gaming() -> Activities {
    [Activity::Gaming, Activity::Recording]
        .into_iter()
        .collect()
}

fn session() -> Vec<Process> {
    vec![
        process(1, 0, "systemd"),
        process(10, 1, "sparky"),
        process(20, 10, "steam"),
        process(30, 20, "reaper"),
        process(31, 30, "ets2"),
        process(40, 10, "obs"),
        process(41, 40, "obs-ffmpeg-mux"),
        process(50, 10, "pipewire"),
        process(60, 10, "rustc"),
    ]
}

#[test]
fn protege_jogo_gravacao_audio_e_o_proprio_desktop() {
    let kept = protected(&session(), 99, &[], &[]);
    for pid in [1, 10, 30, 31, 40, 41, 50, 99] {
        assert!(kept.contains(&pid), "{pid}");
    }
    assert!(!kept.contains(&60));
    assert!(protected(&session(), 99, &["rustc".to_owned()], &[]).contains(&60));
}

#[test]
fn com_jogo_e_cpu_disputada_propoe_aliviar_o_maior_consumidor_de_fundo() {
    let kept = protected(&session(), 99, &[], &[]);
    let consumers = [
        (consumer(31, "ets2"), Usage::cpu(900)),
        (consumer(60, "rustc"), Usage::cpu(400)),
    ];
    let contention = Contention {
        cpu_stall: Some(35.0),
        io_stall: None,
    };
    let Some(candidate) = plan(gaming(), contention, &consumers, &kept, &[5, 11]) else {
        panic!("deveria propor ajuste");
    };
    assert_eq!(candidate.purpose, Purpose::EaseCpu);
    assert_eq!(candidate.name, "rustc");
    assert_eq!(
        candidate
            .options
            .iter()
            .map(|option| option.change.clone())
            .collect::<Vec<_>>(),
        vec![Change::Nice(10), Change::Affinity(vec![5, 11])]
    );
}

#[test]
fn disco_disputado_rebaixa_a_prioridade_de_io() {
    let consumers = [(
        consumer(60, "rustc"),
        Usage {
            cpu: 400,
            io: Some(64 << 20),
        },
    )];
    let contention = Contention {
        cpu_stall: Some(1.0),
        io_stall: Some(40.0),
    };
    let candidate = plan(gaming(), contention, &consumers, &[], &[]);
    assert_eq!(
        candidate.map(|candidate| candidate.options[0].change.clone()),
        Some(Change::IoPriority(IoClass::Idle))
    );
}

#[test]
fn sem_jogo_sem_disputa_ou_sem_consumidor_relevante_nao_interfere() {
    let consumers = [(consumer(60, "rustc"), Usage::cpu(400))];
    let busy = Contention {
        cpu_stall: Some(50.0),
        io_stall: None,
    };
    let desktop: Activities = [Activity::Desktop, Activity::Compiling]
        .into_iter()
        .collect();
    assert_eq!(plan(desktop, busy, &consumers, &[], &[]), None);
    assert_eq!(
        plan(gaming(), Contention::default(), &consumers, &[], &[]),
        None
    );
    assert_eq!(
        plan(
            gaming(),
            busy,
            &[(consumer(60, "rustc"), Usage::cpu(10))],
            &[],
            &[]
        ),
        None
    );
    assert_eq!(plan(gaming(), busy, &consumers, &[60], &[]), None);
}

#[test]
fn a_janela_em_foco_e_seus_filhos_ficam_protegidos() {
    let mut processes = session();
    processes.push(process(70, 10, "lutris"));
    processes.push(process(71, 70, "jogo-nativo"));
    let kept = protected(&processes, 99, &[], &[70]);
    assert!(kept.contains(&70) && kept.contains(&71));
    assert!(!kept.contains(&60));
}

#[test]
fn disco_disputado_escolhe_quem_faz_io_e_nao_quem_gasta_cpu() {
    let consumers = [
        (
            consumer(60, "rustc"),
            Usage {
                cpu: 900,
                io: Some(0),
            },
        ),
        (
            consumer(70, "rsync"),
            Usage {
                cpu: 10,
                io: Some(64 << 20),
            },
        ),
    ];
    let disk = Contention {
        cpu_stall: Some(1.0),
        io_stall: Some(40.0),
    };
    let Some(candidate) = plan(gaming(), disk, &consumers, &[], &[]) else {
        panic!("deveria propor ajuste de disco");
    };
    assert_eq!(candidate.purpose, Purpose::EaseIo);
    assert_eq!(candidate.name, "rsync");
    let cpu = Contention {
        cpu_stall: Some(40.0),
        io_stall: None,
    };
    assert_eq!(
        plan(gaming(), cpu, &consumers, &[], &[]).map(|candidate| candidate.name),
        Some("rustc".to_owned())
    );
}

#[test]
fn sem_telemetria_de_io_nao_ha_ajuste_de_disco() {
    let consumers = [(consumer(70, "rsync"), Usage { cpu: 900, io: None })];
    let disk = Contention {
        cpu_stall: Some(1.0),
        io_stall: Some(40.0),
    };
    assert_eq!(plan(gaming(), disk, &consumers, &[], &[]), None);
}

#[test]
fn pouco_io_nao_justifica_ajuste_de_disco() {
    let consumers = [(
        consumer(70, "rsync"),
        Usage {
            cpu: 10,
            io: Some(1 << 20),
        },
    )];
    let disk = Contention {
        cpu_stall: Some(1.0),
        io_stall: Some(40.0),
    };
    assert_eq!(plan(gaming(), disk, &consumers, &[], &[]), None);
}
