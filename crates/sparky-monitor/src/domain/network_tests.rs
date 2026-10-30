use super::network::{NetCounters, parse_net_dev};

#[test]
fn le_bytes_recebidos_e_enviados_sem_loopback() {
    let text = "Inter-|   Receive |  Transmit\n face |bytes packets|bytes\n    lo: 1231438    8678    0    0    0     0          0         0  1231438    8678    0    0    0     0       0          0\nenp7s0: 908290016  699540    0 2354    0     0          0      4216 46387134  128503    1    9    0     0       0          0\n";
    assert_eq!(
        parse_net_dev(text),
        [NetCounters {
            name: "enp7s0".to_owned(),
            received: 908_290_016,
            sent: 46_387_134
        }]
    );
    assert!(parse_net_dev("a\nb\nwlan0: x y\n").is_empty());
}
