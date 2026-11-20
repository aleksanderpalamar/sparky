use super::admission::{Facts, Owner, admit, owner};
use super::request::{Refusal, Renice};

const STATUS: &str = "Name:\trustc\nUmask:\t0022\nState:\tR (running)\nUid:\t1000\t1000\t1000\t1000\nGid:\t1000\t1000\t1000\t1000\n";

fn request(value: i8) -> Renice {
    Renice {
        task: 900,
        started: 77,
        value,
    }
}

fn facts() -> Facts {
    Facts {
        caller: Some(1000),
        owner: owner(STATUS),
        started: Some(77),
    }
}

fn arguments(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn le_o_dono_real_e_efetivo_do_processo() {
    assert_eq!(
        owner(STATUS),
        Some(Owner {
            real: 1000,
            effective: 1000
        })
    );
    assert_eq!(owner("Name:\tx\n"), None);
    assert_eq!(owner("Uid:\tabc\t1000\n"), None);
}

#[test]
fn os_argumentos_vao_e_voltam_sem_perda() {
    let original = request(0);
    assert_eq!(Renice::parse(&original.arguments()), Ok(original));
    for wrong in [
        arguments(&[]),
        arguments(&["kill", "900", "77", "0"]),
        arguments(&["nice", "900", "77"]),
        arguments(&["nice", "900", "77", "0", "extra"]),
        arguments(&["nice", "-1", "77", "0"]),
        arguments(&["nice", "900", "x", "0"]),
        arguments(&["nice", "900", "77", "300"]),
    ] {
        assert_eq!(Renice::parse(&wrong), Err(Refusal::Usage), "{wrong:?}");
    }
}

#[test]
fn aceita_nice_de_zero_a_dezenove_em_processo_do_proprio_usuario() {
    for value in [0, 10, 19] {
        assert_eq!(admit(&request(value), &facts()), Ok(()));
    }
}

#[test]
fn recusa_nice_fora_da_faixa_permitida() {
    for value in [-20, -1, 20] {
        assert_eq!(admit(&request(value), &facts()), Err(Refusal::OutOfRange));
    }
}

#[test]
fn recusa_processo_de_outro_usuario_ou_com_uid_trocado() {
    let other = Facts {
        caller: Some(1001),
        ..facts()
    };
    assert_eq!(admit(&request(0), &other), Err(Refusal::NotOwner));
    let setuid = Facts {
        owner: Some(Owner {
            real: 1000,
            effective: 0,
        }),
        ..facts()
    };
    assert_eq!(admit(&request(0), &setuid), Err(Refusal::NotOwner));
}

#[test]
fn recusa_root_init_e_chamada_sem_pkexec() {
    let root = Facts {
        caller: Some(0),
        owner: Some(Owner {
            real: 0,
            effective: 0,
        }),
        ..facts()
    };
    assert_eq!(admit(&request(0), &root), Err(Refusal::Protected));
    let direct = Facts {
        caller: None,
        ..facts()
    };
    assert_eq!(admit(&request(0), &direct), Err(Refusal::Protected));
    let init = Renice {
        task: 1,
        ..request(0)
    };
    assert_eq!(admit(&init, &facts()), Err(Refusal::Protected));
}

#[test]
fn recusa_tarefa_que_sumiu_ou_foi_reaproveitada() {
    let reused = Facts {
        started: Some(78),
        ..facts()
    };
    assert_eq!(admit(&request(0), &reused), Err(Refusal::Gone));
    let gone = Facts {
        started: None,
        owner: None,
        ..facts()
    };
    assert_eq!(admit(&request(0), &gone), Err(Refusal::Gone));
}

#[test]
fn cada_recusa_tem_um_codigo_de_saida_proprio() {
    let all = [
        Refusal::Usage,
        Refusal::OutOfRange,
        Refusal::Protected,
        Refusal::NotOwner,
        Refusal::Gone,
        Refusal::Failed,
    ];
    for refusal in all {
        assert_eq!(Refusal::from_code(i32::from(refusal.code())), Some(refusal));
        assert_ne!(refusal.code(), 0);
    }
    assert_eq!(Refusal::from_code(0), None);
    assert_eq!(Refusal::from_code(126), None);
}
