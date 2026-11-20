use super::budget::{Usage, assess, usage};
use crate::application::Budget;
use crate::telemetry::fake_files::FakeFiles;

#[test]
fn le_cpu_e_memoria_do_proprio_processo() {
    let files = FakeFiles::default()
        .with(
            "/proc/self/stat",
            "77 (sparky-resource) S 1 0 0 0 -1 0 0 0 0 0 30 20 0 0 30 10 2 0 1 0",
        )
        .with("/proc/self/status", "Name:\tsparky\nVmRSS:\t    3124 kB\n");
    assert_eq!(
        usage(&files, 100.0),
        Some(Usage {
            cpu_seconds: 0.5,
            rss_kib: 3124
        })
    );
}

#[test]
fn em_jogo_o_teto_de_cpu_e_um_decimo_de_porcento() {
    let before = Usage {
        cpu_seconds: 1.0,
        rss_kib: 3000,
    };
    let calm = Usage {
        cpu_seconds: 1.05,
        rss_kib: 3000,
    };
    assert_eq!(assess(before, calm, 60.0, true), Budget::Within);
    let busy = Usage {
        cpu_seconds: 1.2,
        rss_kib: 3000,
    };
    assert_eq!(assess(before, busy, 60.0, true), Budget::Exceeded);
    assert_eq!(assess(before, busy, 60.0, false), Budget::Within);
}

#[test]
fn memoria_acima_da_meta_normal_degrada() {
    let before = Usage {
        cpu_seconds: 1.0,
        rss_kib: 3000,
    };
    let heavy = Usage {
        cpu_seconds: 1.0,
        rss_kib: 30 * 1024,
    };
    assert_eq!(assess(before, heavy, 60.0, false), Budget::Exceeded);
}
