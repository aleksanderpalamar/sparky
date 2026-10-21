use crate::domain::system::profiles::{self, Profile, label_of};
use crate::domain::system::{Change, Control, Request, Row, RowId};
use crate::domain::value::Step;
use crate::ports::commands::CommandRunner;

const BRIGHTNESS_STEP: u8 = 5;
const DIMMEST: u8 = 1;
const BRIGHTEST: u8 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Battery {
    pub percent: u8,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backlight {
    pub device: String,
    pub percent: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Power {
    pub profiles: Vec<Profile>,
    pub battery: Option<Battery>,
    pub backlight: Option<Backlight>,
}

impl Power {
    pub fn load(commands: &dyn CommandRunner) -> Self {
        let listing = commands.run("powerprofilesctl", &["list"]).unwrap_or_default();
        Self {
            profiles: profiles::parse(&listing),
            battery: battery(commands),
            backlight: backlight(commands),
        }
    }

    pub fn rows(&self) -> Vec<Row> {
        let mut rows = vec![self.profile_row()];
        if let Some(light) = &self.backlight {
            rows.push(Row { id: RowId::Brightness, label: "Brilho da tela".to_owned(), value: format!("{}%", light.percent), control: Control::Level });
        }
        if let Some(battery) = &self.battery {
            let value = format!("{}% · {}", battery.percent, state_label(&battery.state));
            rows.push(Row { id: RowId::Battery, label: "Bateria".to_owned(), value, control: Control::Reading });
        }
        rows
    }

    pub fn request(&self, id: RowId, change: Change) -> Option<Request> {
        match (id, change) {
            (RowId::PowerProfile, Change::Step(step)) => self.neighbor_profile(step),
            (RowId::PowerProfile, Change::Activate) => self.neighbor_profile(Step::More),
            (RowId::Brightness, Change::Step(step)) => {
                let light = self.backlight.as_ref()?;
                let percent = match step {
                    Step::Less => light.percent.saturating_sub(BRIGHTNESS_STEP).max(DIMMEST),
                    Step::More => light.percent.saturating_add(BRIGHTNESS_STEP).min(BRIGHTEST),
                };
                Some(Request::SetBrightness { device: light.device.clone(), percent })
            }
            _ => None,
        }
    }

    fn profile_row(&self) -> Row {
        let active = self.profiles.iter().find(|profile| profile.active);
        match (self.profiles.is_empty(), active) {
            (true, _) => Row {
                id: RowId::PowerProfile,
                label: "Perfil de energia".to_owned(),
                value: "power-profiles-daemon indisponível".to_owned(),
                control: Control::Reading,
            },
            (false, current) => Row {
                id: RowId::PowerProfile,
                label: "Perfil de energia".to_owned(),
                value: current.map_or("nenhum", |profile| label_of(&profile.name)).to_owned(),
                control: Control::Choice,
            },
        }
    }

    fn neighbor_profile(&self, step: Step) -> Option<Request> {
        let count = self.profiles.len();
        let current = self.profiles.iter().position(|profile| profile.active).unwrap_or(0);
        let next = match step {
            Step::Less => (current + count.checked_sub(1)?) % count,
            Step::More => (current + 1) % count,
        };
        let profile = self.profiles.get(next).filter(|profile| !profile.active)?;
        Some(Request::SetProfile(profile.name.clone()))
    }
}

fn battery(commands: &dyn CommandRunner) -> Option<Battery> {
    let devices = commands.run("upower", &["-e"]).ok()?;
    let path = devices.lines().map(str::trim).find(|line| line.contains("battery_"))?;
    let info = commands.run("upower", &["-i", path]).ok()?;
    Some(Battery {
        percent: field(&info, "percentage").as_deref().and_then(percentage)?,
        state: field(&info, "state").unwrap_or_default(),
    })
}

fn backlight(commands: &dyn CommandRunner) -> Option<Backlight> {
    let listing = commands.run("brightnessctl", &["-m", "-c", "backlight", "-l"]).ok()?;
    let line = listing.lines().map(str::trim).find(|line| !line.is_empty())?;
    let mut parts = line.split(',');
    let device = parts.next()?.to_owned();
    let percent = percentage(parts.nth(2)?)?;
    Some(Backlight { device, percent })
}

fn field(info: &str, key: &str) -> Option<String> {
    info.lines()
        .filter_map(|line| line.trim().split_once(':'))
        .map(|(name, value)| (name.trim(), value.trim()))
        .find(|(name, _)| *name == key)
        .map(|(_, value)| value.to_owned())
}

fn percentage(raw: &str) -> Option<u8> {
    raw.trim().trim_end_matches('%').parse().ok()
}

fn state_label(state: &str) -> &str {
    match state {
        "charging" => "carregando",
        "discharging" => "descarregando",
        "fully-charged" => "carregada",
        "pending-charge" => "aguardando carga",
        "empty" => "vazia",
        other => other,
    }
}
