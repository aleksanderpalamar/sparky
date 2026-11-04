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
        let listing = commands
            .run("powerprofilesctl", &["list"])
            .unwrap_or_default();
        Self {
            profiles: profiles::parse(&listing),
            battery: battery(commands),
            backlight: backlight(commands),
        }
    }

    pub fn rows(&self) -> Vec<Row> {
        let mut rows = vec![self.profile_row()];
        if let Some(light) = &self.backlight {
            rows.push(Row {
                id: RowId::Brightness,
                label: crate::i18n::text("row-brightness"),
                value: format!("{}%", light.percent),
                control: Control::Level,
            });
        }
        if let Some(battery) = &self.battery {
            let value = format!("{}% · {}", battery.percent, state_label(&battery.state));
            rows.push(Row {
                id: RowId::Battery,
                label: crate::i18n::text("row-battery"),
                value,
                control: Control::Reading,
            });
        }
        rows
    }

    pub fn request(&self, id: RowId, change: Change) -> Option<Request> {
        match (id, change) {
            (RowId::PowerProfile, Change::Step(step)) => self.neighbor_profile(step),
            (RowId::PowerProfile, Change::Activate) => self.neighbor_profile(Step::More),
            (RowId::PowerProfile, Change::Text(label)) => self.named_profile(&label),
            (RowId::Brightness, Change::Step(step)) => self.stepped_brightness(step),
            (RowId::Brightness, Change::Text(value)) => self.brightness(value.trim().parse().ok()?),
            _ => None,
        }
    }

    fn named_profile(&self, label: &str) -> Option<Request> {
        let profile = self
            .profiles
            .iter()
            .find(|profile| label_of(&profile.name) == label.trim())?;
        (!profile.active).then(|| Request::SetProfile(profile.name.clone()))
    }

    fn stepped_brightness(&self, step: Step) -> Option<Request> {
        let current = self.backlight.as_ref()?.percent;
        self.brightness(match step {
            Step::Less => current.saturating_sub(BRIGHTNESS_STEP),
            Step::More => current.saturating_add(BRIGHTNESS_STEP),
        })
    }

    fn brightness(&self, percent: u8) -> Option<Request> {
        let light = self.backlight.as_ref()?;
        let percent = percent.clamp(DIMMEST, BRIGHTEST);
        (percent != light.percent).then(|| Request::SetBrightness {
            device: light.device.clone(),
            percent,
        })
    }

    fn profile_row(&self) -> Row {
        let active = self.profiles.iter().find(|profile| profile.active);
        match (self.profiles.is_empty(), active) {
            (true, _) => Row {
                id: RowId::PowerProfile,
                label: crate::i18n::text("row-power-profile"),
                value: crate::i18n::text("power-profile-unavailable"),
                control: Control::Reading,
            },
            (false, current) => Row {
                id: RowId::PowerProfile,
                label: crate::i18n::text("row-power-profile"),
                value: current.map_or_else(
                    || crate::i18n::text("none"),
                    |profile| label_of(&profile.name),
                ),
                control: Control::Options(
                    self.profiles
                        .iter()
                        .map(|profile| label_of(&profile.name))
                        .collect(),
                ),
            },
        }
    }

    fn neighbor_profile(&self, step: Step) -> Option<Request> {
        let count = self.profiles.len();
        let current = self
            .profiles
            .iter()
            .position(|profile| profile.active)
            .unwrap_or(0);
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
    let path = devices
        .lines()
        .map(str::trim)
        .find(|line| line.contains("battery_"))?;
    let info = commands.run("upower", &["-i", path]).ok()?;
    Some(Battery {
        percent: field(&info, "percentage").as_deref().and_then(percentage)?,
        state: field(&info, "state").unwrap_or_default(),
    })
}

fn backlight(commands: &dyn CommandRunner) -> Option<Backlight> {
    let listing = commands
        .run("brightnessctl", &["-m", "-c", "backlight", "-l"])
        .ok()?;
    let line = listing
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())?;
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

fn state_label(state: &str) -> String {
    let key = match state {
        "charging" => "battery-charging",
        "discharging" => "battery-discharging",
        "fully-charged" => "battery-full",
        "pending-charge" => "battery-pending",
        "empty" => "battery-empty",
        other => return other.to_owned(),
    };
    crate::i18n::text(key)
}
