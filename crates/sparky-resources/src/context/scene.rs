#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Process {
    pub pid: u32,
    pub parent: u32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    pub app_id: String,
    pub pid: Option<u32>,
    pub focused: bool,
    pub fullscreen: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Load {
    pub cpu_busy_percent: Option<f32>,
    pub memory_available_percent: Option<f32>,
    pub memory_stall_avg10: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Scene {
    pub processes: Option<Vec<Process>>,
    pub windows: Option<Vec<Window>>,
    pub load: Load,
}
