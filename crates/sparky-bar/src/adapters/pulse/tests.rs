use super::commands::valid;
use crate::domain::audio::{AudioRequest, Direction};

#[test]
fn comandos_rejeitam_canais_volume_e_nomes_invalidos_antes_de_chamar_libpulse() {
    let volume = |channels, percent| AudioRequest::Volume {
        direction: Direction::Output,
        name: "fone".to_owned(),
        channels,
        percent,
    };
    assert!(valid(&volume(2, 150)));
    assert!(!valid(&volume(0, 50)));
    assert!(!valid(&volume(33, 50)));
    assert!(!valid(&volume(2, 151)));
    assert!(!valid(&AudioRequest::Default {
        direction: Direction::Input,
        name: String::new()
    }));
    assert!(!valid(&AudioRequest::Mute {
        direction: Direction::Output,
        name: "inva\0lido".to_owned(),
        muted: false
    }));
    assert!(valid(&AudioRequest::AppVolume {
        index: 7,
        channels: 2,
        percent: 100,
    }));
    assert!(!valid(&AudioRequest::AppVolume {
        index: 7,
        channels: 0,
        percent: 100,
    }));
}
