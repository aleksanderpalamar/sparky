use super::consumers::Consumers;
use super::fake_files::FakeFiles;

fn stat(pid: u32, name: &str, ticks: u64, started: u64) -> String {
    format!("{pid} ({name}) R 1 0 0 0 -1 0 0 0 0 0 {ticks} 0 0 0 20 0 1 0 {started} 0")
}

#[test]
fn mede_o_consumo_entre_duas_leituras() {
    let mut consumers = Consumers::default();
    let first = FakeFiles::default().with("/proc/50/stat", &stat(50, "compilador", 100, 7));
    assert_eq!(
        consumers
            .measure(&first, &[50])
            .first()
            .map(|(_, delta)| *delta),
        Some(0)
    );
    let second = FakeFiles::default().with("/proc/50/stat", &stat(50, "compilador", 160, 7));
    assert_eq!(
        consumers
            .measure(&second, &[50])
            .first()
            .map(|(_, delta)| *delta),
        Some(60)
    );
}

#[test]
fn pid_reaproveitado_comeca_do_zero() {
    let mut consumers = Consumers::default();
    let first = FakeFiles::default().with("/proc/50/stat", &stat(50, "velho", 500, 7));
    consumers.measure(&first, &[50]);
    let reused = FakeFiles::default().with("/proc/50/stat", &stat(50, "novo", 900, 8));
    assert_eq!(
        consumers
            .measure(&reused, &[50])
            .first()
            .map(|(_, delta)| *delta),
        Some(0)
    );
}
