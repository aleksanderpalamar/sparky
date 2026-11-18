use super::hardware::fingerprint;
use crate::telemetry::fake_files::FakeFiles;

fn machine(model: &str, memory_kib: u64) -> FakeFiles {
    FakeFiles::default()
        .with(
            "/proc/cpuinfo",
            &format!("processor\t: 0\nmodel name\t: {model}\nprocessor\t: 1\n"),
        )
        .with(
            "/proc/meminfo",
            &format!("MemTotal: {memory_kib} kB\nMemAvailable: 1 kB\n"),
        )
}

#[test]
fn mesma_maquina_tem_a_mesma_impressao() {
    let ryzen = machine("AMD Ryzen 5 5500", 16_149_948);
    assert_eq!(fingerprint(&ryzen, true), fingerprint(&ryzen, true));
}

#[test]
fn trocar_cpu_memoria_ou_gpu_muda_a_impressao() {
    let base = fingerprint(&machine("AMD Ryzen 5 5500", 16_149_948), true);
    assert_ne!(
        base,
        fingerprint(&machine("AMD Ryzen 7 5800X", 16_149_948), true)
    );
    assert_ne!(
        base,
        fingerprint(&machine("AMD Ryzen 5 5500", 32_000_000), true)
    );
    assert_ne!(
        base,
        fingerprint(&machine("AMD Ryzen 5 5500", 16_149_948), false)
    );
}

#[test]
fn oscilacao_pequena_da_memoria_nao_muda_a_impressao() {
    let one = fingerprint(&machine("AMD Ryzen 5 5500", 16_149_948), true);
    assert_eq!(
        one,
        fingerprint(&machine("AMD Ryzen 5 5500", 16_149_000), true)
    );
}
