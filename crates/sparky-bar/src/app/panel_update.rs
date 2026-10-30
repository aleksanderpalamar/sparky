use super::{message::Message, panel_actions, state::Bar};
use crate::domain::{
    bluetooth::BluetoothRequest,
    panel::PanelKind,
    prompt::{Answer, PromptEvent},
};
use iced::Task;

pub fn update(bar: &mut Bar, message: Message) -> Task<Message> {
    match message {
        Message::PanelTab(reverse) if bar.panels.open.is_some() => {
            return match reverse {
                true => iced::widget::operation::focus_previous(),
                false => iced::widget::operation::focus_next(),
            };
        }
        Message::Panel(kind) => return panel_actions::open(bar, kind),
        Message::ClosePanel => return panel_actions::close(bar),
        Message::PanelUnfocused(window)
            if bar.panels.open.is_some_and(|open| open.window == window)
                && bar.panels.input_focus == super::panel_state::InputFocus::Interactive =>
        {
            return panel_actions::close(bar);
        }
        Message::PanelFocused(window)
            if bar.panels.open.is_some_and(|open| open.window == window) =>
        {
            bar.panels.input_focus = super::panel_state::InputFocus::Interactive
        }
        Message::PanelFocusSettled(window)
            if bar.panels.open.is_some_and(|open| open.window == window)
                && bar.panels.input_focus == super::panel_state::InputFocus::Restoring =>
        {
            bar.panels.input_focus = super::panel_state::InputFocus::Interactive
        }
        Message::AudioReady(client) => bar.panels.audio_client = Some(client),
        Message::BluetoothReady(client) => bar.panels.bluetooth_client = Some(client),
        Message::NetworkReady(client) => bar.panels.network_client = Some(client),
        Message::NetworkSample(window, result) => {
            super::panel_network::received(bar, window, result)
        }
        Message::OpenNetworkSettings => return super::panel_network::open_settings(),
        Message::NetworkSettingsOpened(Err(error)) => {
            bar.panels.errors.insert(PanelKind::Network, error);
        }
        Message::PromptReady(client) => bar.panels.prompts = Some(client),
        Message::Audio(audio) => bar.panels.audio = audio,
        Message::Network(network) => bar.panels.network = network,
        Message::AudioRequest(request) => return super::audio::request(bar, request),
        Message::NetworkRequest(request) => return super::network::request(bar, request),
        Message::BluetoothRequest(request) => return bluetooth(bar, request),
        Message::VolumeDraft(direction, percent) => bar.panels.draft_volume(direction, percent),
        Message::VolumeCommit => {
            let Some(request) = bar.panels.volume_request() else {
                return Task::none();
            };
            return super::audio::request(bar, request);
        }
        Message::AppVolumeDraft(index, percent) => bar.panels.draft_app_volume(index, percent),
        Message::AppVolumeCommit(index) => {
            let Some(request) = bar.panels.app_volume_request(index) else {
                return Task::none();
            };
            return super::audio::request(bar, request);
        }
        Message::ChooseWifi(network) => return super::network::choose(bar, network),
        Message::WifiPassword(secret) => {
            if let Some(wifi) = &mut bar.panels.wifi {
                wifi.password = secret;
            }
        }
        Message::WifiRemember(remember) => {
            if let Some(wifi) = &mut bar.panels.wifi {
                wifi.remember = remember;
            }
        }
        Message::WifiConnect => {
            let Some(request) = bar.panels.wifi_request() else {
                return Task::none();
            };
            return super::network::request(bar, request);
        }
        Message::WifiCancel => bar.panels.wifi = None,
        Message::Prompt(event) => return prompt(bar, event),
        Message::PromptInput(secret) => bar.panels.prompt_input = secret,
        Message::PromptAnswer(answer) => return answer_prompt(bar, answer),
        Message::OperationFinished(id, kind, result) => return finish(bar, id, kind, result),
        _ => {}
    }
    Task::none()
}

fn prompt(bar: &mut Bar, event: PromptEvent) -> Task<Message> {
    match event {
        PromptEvent::Dismiss(id) => bar.panels.dismiss(id),
        PromptEvent::Show(prompt) => {
            let valid = bar.panels.open.is_some_and(|open| {
                open.kind == prompt.kind.panel()
                    && bar
                        .panels
                        .pending
                        .get(&open.kind)
                        .is_some_and(|pending| pending.origin == Some(open.window))
            });
            if !valid {
                return reject_prompt(bar, prompt.id);
            }
            bar.panels.prompt_input = Default::default();
            bar.panels.prompt = Some(prompt);
            return panel_actions::restore_keyboard(
                bar,
                bar.panels.open.map_or(PanelKind::Network, |open| open.kind),
            );
        }
    }
    Task::none()
}

fn reject_prompt(bar: &Bar, id: crate::domain::prompt::PromptId) -> Task<Message> {
    let Some(client) = &bar.panels.prompts else {
        return Task::none();
    };
    client.0.answer(id, Answer::Reject);
    Task::none()
}

fn answer_prompt(bar: &mut Bar, answer: Answer) -> Task<Message> {
    let Some(prompt) = bar.panels.prompt.take() else {
        return Task::none();
    };
    let Some(client) = &bar.panels.prompts else {
        return Task::none();
    };
    client.0.answer(prompt.id, answer);
    bar.panels.prompt_input = Default::default();
    panel_actions::yield_keyboard(bar, prompt.kind.panel())
}

fn finish(bar: &mut Bar, id: u64, kind: PanelKind, result: Result<(), String>) -> Task<Message> {
    let current = bar.panels.pending.get(&kind).is_some_and(|pending| {
        pending.id == id && pending.origin == bar.panels.open.map(|open| open.window)
    });
    bar.panels.finish(id, kind, result);
    if !current {
        return Task::none();
    }
    panel_actions::restore_keyboard(bar, kind)
}

pub fn bluetooth(bar: &mut Bar, request: BluetoothRequest) -> Task<Message> {
    let Some(client) = bar.panels.bluetooth_client.clone() else {
        return Task::none();
    };
    if request == BluetoothRequest::Cancel {
        return Task::perform(async move { client.0.send(request).await }, |_| {
            Message::Ignore
        });
    }
    if bar.panels.pending.contains_key(&PanelKind::Bluetooth) {
        return Task::none();
    }
    let id = bar.panels.begin(PanelKind::Bluetooth);
    let operation = Task::perform(async move { client.0.send(request).await }, move |result| {
        Message::OperationFinished(id, PanelKind::Bluetooth, result)
    });
    Task::batch([
        panel_actions::yield_keyboard(bar, PanelKind::Bluetooth),
        operation,
    ])
}
