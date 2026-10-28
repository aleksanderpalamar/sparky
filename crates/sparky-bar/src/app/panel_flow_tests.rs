use super::{
    message::Message,
    panel_state::OpenPanel,
    tray_test_support::{bar, messages, removes},
    update::update,
};
use crate::domain::{
    audio::{Audio, AudioDevice, AudioRequest, AudioState, Direction},
    network::{Security, WifiNetwork},
    panel::PanelKind,
    prompt::{Prompt, PromptEvent, PromptId, PromptKind},
    secret::Secret,
};

pub fn wifi() -> WifiNetwork {
    WifiNetwork {
        path: "/ap/1".to_owned(),
        device: "/wifi/1".to_owned(),
        ssid: b"Minha rede".to_vec(),
        strength: 80,
        security: Security::Personal,
        profile: None,
        active: false,
    }
}

#[test]
fn painel_alterna_fecha_anterior_e_limpa_senha_ao_fechar() {
    let mut bar = bar();
    let _ = messages(update(&mut bar, Message::Panel(PanelKind::Network)));
    let first = bar.panels.open.unwrap().window;
    let _ = update(&mut bar, Message::ChooseWifi(wifi()));
    let _ = update(
        &mut bar,
        Message::WifiPassword(Secret::new("12345678".to_owned())),
    );
    assert!(bar.panels.wifi.is_some());
    let output = messages(update(&mut bar, Message::Panel(PanelKind::Audio)));
    assert!(removes(&output, first));
    assert!(bar.panels.wifi.is_none());
    let second = bar.panels.open.unwrap().window;
    let output = messages(update(&mut bar, Message::Panel(PanelKind::Audio)));
    assert!(removes(&output, second));
    assert!(bar.panels.open.is_none());
}

#[test]
fn foco_perdido_de_outra_janela_nao_fecha_o_painel() {
    let mut bar = bar();
    let _ = messages(update(&mut bar, Message::Panel(PanelKind::Network)));
    let window = bar.panels.open.unwrap().window;
    let _ = update(
        &mut bar,
        Message::PanelUnfocused(iced::window::Id::unique()),
    );
    assert_eq!(bar.panels.open.unwrap().window, window);
    assert!(removes(
        &messages(update(&mut bar, Message::PanelUnfocused(window))),
        window
    ));
}

#[test]
fn conclusao_antiga_nao_altera_operacao_nova_ou_painel_reaberto() {
    let mut bar = bar();
    let _ = messages(update(&mut bar, Message::Panel(PanelKind::Network)));
    let first = bar.panels.begin(PanelKind::Network);
    let second = bar.panels.begin(PanelKind::Network);
    bar.panels
        .finish(first, PanelKind::Network, Err("antiga".to_owned()));
    assert_eq!(bar.panels.pending[&PanelKind::Network].id, second);
    let _ = messages(update(&mut bar, Message::ClosePanel));
    let _ = messages(update(&mut bar, Message::Panel(PanelKind::Network)));
    bar.panels
        .finish(second, PanelKind::Network, Err("outra janela".to_owned()));
    assert!(bar.panels.errors.is_empty());
}

#[test]
fn slider_envia_valor_final_para_dispositivo_correto_e_limita_amplificacao() {
    let mut bar = bar();
    bar.panels.audio = Audio::Ready(AudioState {
        outputs: vec![AudioDevice {
            name: "fone".to_owned(),
            label: "Fone".to_owned(),
            channels: 2,
            volume: 50,
            muted: false,
        }],
        default_output: Some("fone".to_owned()),
        ..Default::default()
    });
    let _ = update(&mut bar, Message::VolumeDraft(Direction::Output, 300));
    assert_eq!(
        bar.panels.volume_request(),
        Some(AudioRequest::Volume {
            direction: Direction::Output,
            name: "fone".to_owned(),
            channels: 2,
            percent: 150
        })
    );
    let _ = update(&mut bar, Message::VolumeDraft(Direction::Input, 80));
    assert_eq!(bar.panels.volume.as_ref().unwrap().name, "fone");
}

#[test]
fn solicitacoes_tardias_nao_abrem_formulario_em_nova_janela() {
    let mut bar = bar();
    let old = iced::window::Id::unique();
    bar.panels.open = Some(OpenPanel {
        window: old,
        kind: PanelKind::Network,
    });
    bar.panels.begin(PanelKind::Network);
    bar.panels.open = Some(OpenPanel {
        window: iced::window::Id::unique(),
        kind: PanelKind::Network,
    });
    let _ = update(
        &mut bar,
        Message::Prompt(PromptEvent::Show(Prompt {
            id: PromptId(1),
            kind: PromptKind::Wifi {
                ssid: "rede".to_owned(),
            },
        })),
    );
    assert!(bar.panels.prompt.is_none());
}
