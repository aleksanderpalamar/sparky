use sparky_ipc::OutputInfo;

use super::diagram::{Placement, map};

fn monitor(name: &str, x: i32, width: i32, height: i32, focused: bool) -> OutputInfo {
    OutputInfo {
        name: name.to_owned(),
        x,
        y: 0,
        width,
        height,
        focused,
    }
}

#[test]
fn um_monitor_fica_centralizado_e_proporcional() {
    let places = map(&[monitor("HDMI-A-1", 0, 1920, 1080, true)], 424.0, 176.0);
    let Some(place) = places.first() else {
        panic!("sem monitor")
    };
    assert_eq!(place.number, 1);
    assert!(place.focused);
    assert!((place.height - 150.0).abs() < 0.01);
    assert!((place.width - (1920.0 * 152.0 / 1080.0 - 2.0)).abs() < 0.01);
    assert!((place.x + place.width / 2.0 + 1.0 - 212.0).abs() < 0.01);
}

#[test]
fn monitores_lado_a_lado_mantem_ordem_e_posicao() {
    let places = map(
        &[
            monitor("DP-1", 1920, 2560, 1440, false),
            monitor("HDMI-A-1", 0, 1920, 1080, true),
        ],
        700.0,
        176.0,
    );
    let numbers: Vec<_> = places
        .iter()
        .map(|place: &Placement| (place.number, place.focused))
        .collect();
    assert_eq!(numbers, [(1, false), (2, true)]);
    assert!(places[1].x < places[0].x);
    assert!(places[0].height > places[1].height);
}

#[test]
fn sem_monitores_nao_ha_desenho() {
    assert!(map(&[], 400.0, 176.0).is_empty());
}
