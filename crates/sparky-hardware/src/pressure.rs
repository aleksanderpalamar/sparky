#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Stall {
    pub avg10: f32,
    pub avg60: f32,
    pub avg300: f32,
    pub total_us: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pressure {
    pub some: Stall,
    pub full: Option<Stall>,
}

pub fn parse_pressure(text: &str) -> Option<Pressure> {
    let mut some = None;
    let mut full = None;
    for line in text.lines() {
        let (kind, values) = line.split_once(' ')?;
        match kind {
            "some" => some = Some(stall(values)?),
            "full" => full = Some(stall(values)?),
            _ => return None,
        }
    }
    Some(Pressure { some: some?, full })
}

fn stall(values: &str) -> Option<Stall> {
    let field = |key: &str| {
        values
            .split_whitespace()
            .find_map(|pair| pair.strip_prefix(key)?.strip_prefix('='))
    };
    Some(Stall {
        avg10: field("avg10")?.parse().ok()?,
        avg60: field("avg60")?.parse().ok()?,
        avg300: field("avg300")?.parse().ok()?,
        total_us: field("total")?.parse().ok()?,
    })
}
