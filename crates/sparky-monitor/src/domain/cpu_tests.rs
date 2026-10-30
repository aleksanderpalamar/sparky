use super::cpu::{CpuTimes, cpu_sensor, mean_frequency, model, parse_stat, usage};

const STAT: &str = "cpu  100 0 100 700 100 0 0 0 0 0\ncpu0 50 0 50 350 50 0 0 0 0 0\ncpu1 50 0 50 350 50 0 0 0 0 0\nintr 1 2 3\n";

#[test]
fn le_total_e_nucleos_separando_ocioso_de_ocupado() {
    let (total, cores) = parse_stat(STAT).unwrap();
    assert_eq!(
        total,
        CpuTimes {
            busy: 200,
            total: 1000
        }
    );
    assert_eq!(cores.len(), 2);
    assert_eq!(
        cores[1],
        CpuTimes {
            busy: 100,
            total: 500
        }
    );
}

#[test]
fn uso_e_a_fracao_ocupada_entre_duas_leituras() {
    let antes = CpuTimes {
        busy: 200,
        total: 1000,
    };
    let depois = CpuTimes {
        busy: 260,
        total: 1200,
    };
    assert_eq!(usage(antes, depois), 30.0);
    assert_eq!(usage(depois, depois), 0.0);
    assert_eq!(usage(depois, antes), 0.0);
}

#[test]
fn modelo_frequencia_media_e_sensor() {
    let cpuinfo = "processor\t: 0\nmodel name\t: AMD Ryzen 5 5500\ncpu MHz\t\t: 3600.000\n";
    assert_eq!(model(cpuinfo).as_deref(), Some("AMD Ryzen 5 5500"));
    assert_eq!(model("processor\t: 0\n"), None);
    assert_eq!(mean_frequency(&[3_600_000, 2_400_000]), Some(3000));
    assert_eq!(mean_frequency(&[]), None);
    assert!(cpu_sensor("k10temp"));
    assert!(cpu_sensor("coretemp"));
    assert!(!cpu_sensor("nvme"));
}

#[test]
fn texto_estranho_nao_derruba_a_leitura() {
    assert_eq!(parse_stat("intr 1\n"), None);
    assert!(parse_stat("cpu  a b c\n").is_none());
}
