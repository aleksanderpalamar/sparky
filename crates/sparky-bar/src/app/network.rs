use super::{message::Message, panel_state::WifiPassword, state::Bar};
use crate::{
    adapters::icons::Icons,
    domain::{
        indicator::{Role, Status},
        network::{ADDRESS, NetworkRequest, Security, WifiNetwork},
        panel::PanelKind,
        secret::Secret,
        tray::{PrimaryAction, TrayIcon, TrayItem},
    },
};
use iced::Task;

pub fn indicator(state: &crate::domain::network::Network, icons: &Icons) -> TrayItem {
    let (icon, title) = match state {
        crate::domain::network::Network::Ready(state) => (state.icon(), "Rede".to_owned()),
        crate::domain::network::Network::Unavailable(error) => {
            ("network-offline-symbolic", error.clone())
        }
    };
    TrayItem {
        address: ADDRESS.to_owned(),
        title,
        icon: icons.resolve(icon, None).unwrap_or(TrayIcon::Missing),
        role: Role::Network,
        status: Status::Active,
        primary_action: PrimaryAction::Activate,
        menu_path: None,
        menu: Vec::new(),
    }
}

pub fn request(bar: &mut Bar, request: NetworkRequest) -> Task<Message> {
    let Some(client) = bar.panels.network_client.clone() else {
        return Task::none();
    };
    if bar.panels.pending.contains_key(&PanelKind::Network) {
        return Task::none();
    }
    let id = bar.panels.begin(PanelKind::Network);
    bar.panels.wifi = None;
    let operation = Task::perform(async move { client.0.send(request).await }, move |result| {
        Message::OperationFinished(id, PanelKind::Network, result)
    });
    Task::batch([
        super::panel_actions::yield_keyboard(bar, PanelKind::Network),
        operation,
    ])
}

pub fn choose(bar: &mut Bar, network: WifiNetwork) -> Task<Message> {
    if network.active {
        return request(bar, NetworkRequest::Disconnect(network.device));
    }
    if let Some(profile) = &network.profile {
        return request(
            bar,
            NetworkRequest::Activate {
                profile: profile.clone(),
                device: network.device,
                access_point: network.path,
            },
        );
    }
    match network.security {
        Security::Open => request(
            bar,
            NetworkRequest::Join {
                network,
                password: Secret::default(),
                remember: true,
            },
        ),
        Security::Personal | Security::Sae => {
            bar.panels.wifi = Some(WifiPassword {
                network,
                password: Secret::default(),
                remember: true,
            });
            Task::none()
        }
        _ => {
            bar.panels.errors.insert(PanelKind::Network, "Esta rede exige um perfil previamente configurado; o painel aceita novas redes abertas, WPA2-Personal e WPA3-Personal".to_owned());
            Task::none()
        }
    }
}
