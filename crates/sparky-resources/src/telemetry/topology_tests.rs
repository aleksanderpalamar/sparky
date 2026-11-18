use super::fake_files::FakeFiles;
use super::topology::read_topology;

fn cpu(files: FakeFiles, id: u32, core: u32, siblings: &str) -> FakeFiles {
    let base = format!("/sys/devices/system/cpu/cpu{id}");
    files
        .with(&format!("{base}/topology/core_id"), &format!("{core}\n"))
        .with(&format!("{base}/topology/physical_package_id"), "0\n")
        .with(&format!("{base}/topology/thread_siblings_list"), siblings)
        .with(&format!("{base}/cache/index0/shared_cpu_list"), siblings)
        .with(&format!("{base}/cache/index3/shared_cpu_list"), "0-3\n")
}

fn ryzen_like() -> FakeFiles {
    let files = FakeFiles::default().with("/sys/devices/system/cpu/online", "0-3\n");
    let files = cpu(files, 0, 0, "0,2\n");
    let files = cpu(files, 1, 1, "1,3\n");
    let files = cpu(files, 2, 0, "0,2\n");
    cpu(files, 3, 1, "1,3\n")
}

#[test]
fn distingue_nucleos_fisicos_de_threads_logicas() {
    let topology = read_topology(&ryzen_like());
    assert_eq!(
        topology.as_ref().map(|topology| topology.logical()),
        Some(4)
    );
    assert_eq!(
        topology.as_ref().map(|topology| topology.physical()),
        Some(2)
    );
    assert_eq!(
        topology.as_ref().map(|topology| topology.has_smt()),
        Some(true)
    );
    assert_eq!(
        topology
            .as_ref()
            .map(|topology| topology.siblings(2).to_vec()),
        Some(vec![0, 2])
    );
}

#[test]
fn usa_o_ultimo_nivel_de_cache_compartilhado() {
    let topology = read_topology(&ryzen_like());
    let last =
        topology.and_then(|topology| topology.cpus.first().map(|cpu| cpu.last_cache.clone()));
    assert_eq!(last, Some(vec![0, 1, 2, 3]));
}

#[test]
fn sem_smt_cada_thread_e_um_nucleo() {
    let files = FakeFiles::default().with("/sys/devices/system/cpu/online", "0-1\n");
    let files = cpu(cpu(files, 0, 0, "0\n"), 1, 1, "1\n");
    let topology = read_topology(&files);
    assert_eq!(
        topology.as_ref().map(|topology| topology.has_smt()),
        Some(false)
    );
    assert_eq!(topology.map(|topology| topology.physical()), Some(2));
}

#[test]
fn topologia_incompleta_nao_e_inventada() {
    let files = FakeFiles::default().with("/sys/devices/system/cpu/online", "0-1\n");
    assert_eq!(read_topology(&cpu(files, 0, 0, "0\n")), None);
}
