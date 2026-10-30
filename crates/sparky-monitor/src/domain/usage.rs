use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Usage {
    pub cpu: f32,
    pub memory: u64,
    pub disk: f64,
    pub gpu: Option<f32>,
    pub vram: Option<u64>,
}

impl Add for Usage {
    type Output = Usage;

    fn add(self, other: Usage) -> Usage {
        Usage {
            cpu: self.cpu + other.cpu,
            memory: self.memory + other.memory,
            disk: self.disk + other.disk,
            gpu: sum(self.gpu, other.gpu),
            vram: sum(self.vram, other.vram),
        }
    }
}

fn sum<T: Add<Output = T>>(left: Option<T>, right: Option<T>) -> Option<T> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left + right),
        (left, right) => left.or(right),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessRow {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub program: String,
    pub user: Option<u32>,
    pub usage: Usage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppRow {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub pids: Vec<u32>,
    pub usage: Usage,
}
