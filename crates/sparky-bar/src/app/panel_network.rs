use iced::Task;

use crate::app::{message::Message, state::Bar};
use crate::domain::network::Network;
use crate::domain::panel::PanelKind;
use crate::domain::telemetry::Metrics;

pub fn sample(bar: &mut Bar) -> Task<Message> {
    let Some(open) = bar
        .panels
        .open
        .filter(|open| open.kind == PanelKind::Network)
    else {
        return Task::none();
    };
    if bar.panels.sampling {
        return Task::none();
    }
    let Network::Ready(state) = &bar.panels.network else {
        return Task::none();
    };
    let Some(device) = state.devices.iter().find(|device| device.state == 100) else {
        bar.panels.metrics = None;
        return Task::none();
    };
    let interface = device.interface.clone();
    let gateway = device
        .gateway
        .as_deref()
        .and_then(|gateway| gateway.parse().ok());
    bar.panels.sampling = true;
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                crate::adapters::telemetry::sample(interface, gateway)
            })
            .await
            .map_err(|error| error.to_string())?
        },
        move |result| Message::NetworkSample(open.window, result),
    )
}

pub fn received(
    bar: &mut Bar,
    window: iced::window::Id,
    result: Result<crate::domain::telemetry::Sample, String>,
) {
    if bar
        .panels
        .open
        .is_none_or(|open| open.window != window || open.kind != PanelKind::Network)
    {
        return;
    }
    bar.panels.sampling = false;
    let Ok(sample) = result else {
        bar.panels.metrics = None;
        return;
    };
    match &mut bar.panels.metrics {
        Some(metrics) => metrics.update(sample),
        None => bar.panels.metrics = Some(Metrics::begin(sample)),
    }
}

pub fn open_settings() -> Task<Message> {
    Task::perform(
        async {
            std::process::Command::new("sparky-settings")
                .args(["--section", "network"])
                .spawn()
                .map(|_| ())
                .map_err(|error| format!("Não foi possível abrir Configurações de rede: {error}"))
        },
        Message::NetworkSettingsOpened,
    )
}
