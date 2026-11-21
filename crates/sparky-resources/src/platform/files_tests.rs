use super::files::LinuxFiles;
use crate::telemetry::{Collector, read_topology};

#[test]
fn le_cpu_e_memoria_do_linux_real() {
    let sample = Collector::new(LinuxFiles).system();
    assert!(sample.cpu.measured().is_some());
    assert!(!sample.cores.is_empty());
    assert!(
        sample
            .memory
            .measured()
            .is_some_and(|memory| memory.total_kib > 0)
    );
}

#[test]
fn le_a_topologia_real_com_pelo_menos_um_nucleo() {
    let topology = read_topology(&LinuxFiles);
    assert!(
        topology
            .as_ref()
            .is_some_and(|topology| topology.logical() >= topology.physical())
    );
    assert!(topology.is_some_and(|topology| topology.physical() > 0));
}
