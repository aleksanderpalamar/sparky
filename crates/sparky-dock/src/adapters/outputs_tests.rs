use sparky_ipc::OutputInfo;

use super::outputs::width;

fn output(name: &str, width: i32, focused: bool, enabled: bool) -> OutputInfo {
    OutputInfo {
        name: name.to_owned(),
        width,
        focused,
        enabled,
        ..OutputInfo::default()
    }
}

#[test]
fn usa_a_largura_do_monitor_focado() {
    let outputs = [
        output("DP-1", 2560, false, true),
        output("HDMI-A-1", 1366, true, true),
    ];
    assert_eq!(width(&outputs), Some(1366));
}

#[test]
fn sem_foco_usa_o_primeiro_ligado_e_ignora_os_desligados() {
    let outputs = [
        output("DP-1", 1920, false, false),
        output("DP-2", 1600, false, true),
    ];
    assert_eq!(width(&outputs), Some(1600));
    assert_eq!(width(&[]), None);
}
