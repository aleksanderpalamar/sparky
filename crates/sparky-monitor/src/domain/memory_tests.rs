use super::memory::Memory;

const MEMINFO: &str = "MemTotal:       16150580 kB\nMemFree:  100 kB\nMemAvailable:    8563588 kB\nSwapTotal:       4194300 kB\nSwapFree:            140 kB\n";

#[test]
fn usado_e_o_total_menos_o_disponivel() {
    let memory = Memory::parse(MEMINFO).unwrap();
    assert_eq!(memory.total, 16_150_580 * 1024);
    assert_eq!(memory.used(), (16_150_580 - 8_563_588) * 1024);
    assert_eq!(format!("{:.1}", memory.percent()), "47.0");
    assert_eq!(memory.swap_used(), (4_194_300 - 140) * 1024);
}

#[test]
fn sem_total_nao_ha_leitura_e_swap_ausente_vale_zero() {
    assert_eq!(Memory::parse("MemAvailable: 10 kB\n"), None);
    let memory = Memory::parse("MemTotal: 100 kB\nMemAvailable: 40 kB\n").unwrap();
    assert_eq!(memory.swap_total, 0);
    assert_eq!(memory.swap_used(), 0);
}
