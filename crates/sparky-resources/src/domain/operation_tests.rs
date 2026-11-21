use super::operation::{Change, IoClass};

#[test]
fn o_resumo_do_ajuste_e_curto_e_legivel() {
    assert_eq!(Change::Nice(10).summary(), "nice 10");
    assert_eq!(Change::Affinity(vec![5, 11]).summary(), "CPU 5, 11");
    assert_eq!(
        Change::IoPriority(IoClass::BestEffort(7)).summary(),
        "io best-effort 7"
    );
    assert_eq!(Change::IoPriority(IoClass::Idle).summary(), "io idle");
    assert_eq!(Change::IoPriority(IoClass::None).summary(), "io none");
}
