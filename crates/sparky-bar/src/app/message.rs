use iced_layershell::to_layer_message;

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    WorkspacesChanged,
    Tick,
    FocusWorkspace(u32),
    ToggleClock,
}
