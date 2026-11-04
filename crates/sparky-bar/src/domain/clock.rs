use chrono::{Datelike, NaiveDateTime, Timelike};
use fluent_bundle::FluentArgs;

const WEEKDAYS: [&str; 7] = [
    "weekday-monday",
    "weekday-tuesday",
    "weekday-wednesday",
    "weekday-thursday",
    "weekday-friday",
    "weekday-saturday",
    "weekday-sunday",
];
const MONTHS: [&str; 12] = [
    "month-january",
    "month-february",
    "month-march",
    "month-april",
    "month-may",
    "month-june",
    "month-july",
    "month-august",
    "month-september",
    "month-october",
    "month-november",
    "month-december",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClockFormat {
    #[default]
    Time,
    Date,
}

impl ClockFormat {
    pub fn toggled(self) -> Self {
        match self {
            Self::Time => Self::Date,
            Self::Date => Self::Time,
        }
    }
}

pub fn render(format: ClockFormat, moment: &NaiveDateTime) -> String {
    let mut args = FluentArgs::new();
    args.set("weekday", weekday(moment));
    args.set("day", format!("{:02}", moment.day()));
    args.set("month", month(moment.month0()));
    args.set("year", moment.year().to_string());
    args.set("hour", format!("{:02}", moment.hour()));
    args.set("minute", format!("{:02}", moment.minute()));
    let key = match format {
        ClockFormat::Time => "clock-time",
        ClockFormat::Date => "clock-date",
    };
    crate::i18n::message(key, Some(&args))
}

fn weekday(moment: &NaiveDateTime) -> String {
    crate::i18n::text(WEEKDAYS[moment.weekday().num_days_from_monday() as usize])
}

pub(super) fn month(index: u32) -> String {
    crate::i18n::text(MONTHS[index as usize])
}

#[cfg(test)]
mod tests {
    use super::{ClockFormat, render};
    use chrono::NaiveDate;

    fn momento(ano: i32, mes: u32, dia: u32, hora: u32, minuto: u32) -> chrono::NaiveDateTime {
        let Some(data) = NaiveDate::from_ymd_opt(ano, mes, dia) else {
            panic!("data inválida");
        };
        let Some(momento) = data.and_hms_opt(hora, minuto, 0) else {
            panic!("hora inválida");
        };
        momento
    }

    #[test]
    fn mostra_o_dia_da_semana_e_a_hora() {
        assert_eq!(
            render(ClockFormat::Time, &momento(2026, 10, 7, 9, 5)),
            "quarta 09:05"
        );
    }

    #[test]
    fn mostra_a_data_por_extenso() {
        assert_eq!(
            render(ClockFormat::Date, &momento(2026, 10, 7, 9, 5)),
            "quarta, 07 de outubro de 2026"
        );
    }

    #[test]
    fn domingo_e_o_ultimo_dia_da_semana() {
        assert_eq!(
            render(ClockFormat::Time, &momento(2026, 10, 18, 23, 59)),
            "domingo 23:59"
        );
        assert_eq!(
            render(ClockFormat::Time, &momento(2026, 10, 19, 0, 0)),
            "segunda 00:00"
        );
    }

    #[test]
    fn janeiro_e_dezembro_nas_pontas() {
        assert_eq!(
            render(ClockFormat::Date, &momento(2027, 1, 1, 0, 0)),
            "sexta, 01 de janeiro de 2027"
        );
        assert_eq!(
            render(ClockFormat::Date, &momento(2026, 12, 31, 0, 0)),
            "quinta, 31 de dezembro de 2026"
        );
    }

    #[test]
    fn alternar_vai_e_volta() {
        assert_eq!(ClockFormat::Time.toggled(), ClockFormat::Date);
        assert_eq!(ClockFormat::Date.toggled(), ClockFormat::Time);
    }
}
