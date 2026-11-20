use crate::cpulist::parse_cpu_list;
use crate::pressure::{Stall, parse_pressure};
use crate::process::parse_stat as parse_process;
use crate::stat::{CpuTimes, parse_stat, usage};

const PSI: &str = "some avg10=1.50 avg60=0.25 avg300=0.00 total=2339028\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=12\n";

#[test]
fn le_pressao_parcial_e_total() {
    let pressure = parse_pressure(PSI);
    assert_eq!(
        pressure.map(|pressure| pressure.some),
        Some(Stall {
            avg10: 1.5,
            avg60: 0.25,
            avg300: 0.0,
            total_us: 2_339_028
        })
    );
    assert_eq!(
        pressure
            .and_then(|pressure| pressure.full)
            .map(|full| full.total_us),
        Some(12)
    );
}

#[test]
fn pressao_so_com_linha_parcial_ou_invalida() {
    let only_some = parse_pressure("some avg10=0.00 avg60=0.00 avg300=0.00 total=1\n");
    assert_eq!(only_some.and_then(|pressure| pressure.full), None);
    assert!(only_some.is_some());
    assert_eq!(
        parse_pressure("full avg10=0.00 avg60=0.00 avg300=0.00 total=1\n"),
        None
    );
    assert_eq!(
        parse_pressure("some avg10=x avg60=0 avg300=0 total=1\n"),
        None
    );
}

#[test]
fn lista_de_cpus_aceita_faixas_e_avulsos() {
    assert_eq!(parse_cpu_list("0-2,6\n"), Some(vec![0, 1, 2, 6]));
    assert_eq!(parse_cpu_list("0,6"), Some(vec![0, 6]));
    assert_eq!(parse_cpu_list(""), Some(vec![]));
    assert_eq!(parse_cpu_list("3-1"), None);
    assert_eq!(parse_cpu_list("a"), None);
}

#[test]
fn tempo_de_cpu_separa_ocioso_e_gera_uso() {
    let stat = "cpu  100 0 100 700 100 0 0 0 0 0\ncpu0 100 0 100 700 100 0 0 0 0 0\n";
    let before = parse_stat(stat).map(|(total, _)| total);
    assert_eq!(
        before,
        Some(CpuTimes {
            busy: 200,
            total: 1000
        })
    );
    let after = CpuTimes {
        busy: 260,
        total: 1200,
    };
    assert_eq!(before.map(|before| usage(before, after)), Some(30.0));
}

#[test]
fn processo_guarda_o_inicio_para_identidade() {
    let text = "77 (obs (studio)) S 1 0 0 0 -1 0 0 0 0 0 9 1 0 0 20 0 1 0 4242 0";
    let process = parse_process(text);
    assert_eq!(
        process.as_ref().map(|process| process.comm.as_str()),
        Some("obs (studio)")
    );
    assert_eq!(process.as_ref().map(|process| process.started), Some(4242));
    assert_eq!(process.map(|process| process.ticks), Some(10));
}
