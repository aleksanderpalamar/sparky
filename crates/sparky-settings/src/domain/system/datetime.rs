use crate::domain::system::key_value;
use crate::domain::system::{Change, Control, Request, Row, RowId};
use crate::ports::commands::CommandRunner;

const UNSET: &str = "não definido";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DateTime {
    timezone: Option<String>,
    local_time: Option<String>,
    network_time: bool,
    synchronized: bool,
    can_network_time: bool,
    local_clock: bool,
    timezones: Vec<String>,
}

impl DateTime {
    pub fn load(commands: &dyn CommandRunner) -> Self {
        let show = commands.run("timedatectl", &["show"]).unwrap_or_default();
        let zones = commands.run("timedatectl", &["list-timezones"]).unwrap_or_default();
        Self {
            timezone: key_value::value(&show, "Timezone"),
            local_time: key_value::value(&show, "TimeUSec"),
            network_time: key_value::flag(&show, "NTP"),
            synchronized: key_value::flag(&show, "NTPSynchronized"),
            can_network_time: key_value::flag(&show, "CanNTP"),
            local_clock: key_value::flag(&show, "LocalRTC"),
            timezones: zones.lines().map(str::trim).filter(|zone| !zone.is_empty()).map(str::to_owned).collect(),
        }
    }

    pub fn rows(&self) -> Vec<Row> {
        vec![
            Row {
                id: RowId::Timezone,
                label: "Fuso horário".to_owned(),
                value: self.timezone.as_deref().unwrap_or(UNSET).to_owned(),
                control: Control::Text,
            },
            Row {
                id: RowId::LocalTime,
                label: "Hora local".to_owned(),
                value: self.local_time.as_deref().unwrap_or(UNSET).to_owned(),
                control: Control::Reading,
            },
            Row {
                id: RowId::NetworkTime,
                label: "Hora pela rede".to_owned(),
                value: self.synchronization_label().to_owned(),
                control: match self.can_network_time {
                    true => Control::Toggle(self.network_time),
                    false => Control::Reading,
                },
            },
            Row {
                id: RowId::LocalClock,
                label: "Relógio do hardware em hora local".to_owned(),
                value: match self.local_clock {
                    true => "ligado",
                    false => "desligado",
                }.to_owned(),
                control: Control::Toggle(self.local_clock),
            },
        ]
    }

    pub fn request(&self, id: RowId, change: Change) -> Option<Request> {
        match (id, change) {
            (RowId::Timezone, Change::Text(input)) if self.unknown_timezone(&input).is_none() => {
                Some(Request::SetTimezone(input.trim().to_owned()))
            }
            (RowId::NetworkTime, Change::Activate | Change::Step(_)) if self.can_network_time => {
                Some(Request::SetNetworkTime(!self.network_time))
            }
            (RowId::LocalClock, Change::Activate | Change::Step(_)) => Some(Request::SetLocalClock(!self.local_clock)),
            _ => None,
        }
    }

    pub fn unknown_timezone(&self, input: &str) -> Option<String> {
        let timezone = input.trim();
        (!self.timezones.iter().any(|known| known == timezone)).then(|| timezone.to_owned())
    }

    fn synchronization_label(&self) -> &'static str {
        match (self.network_time, self.synchronized) {
            (false, _) => "desligada",
            (true, false) => "ligada · sincronizando",
            (true, true) => "ligada · sincronizado",
        }
    }
}
