use chrono::{Datelike, NaiveDateTime, Timelike};

const WEEKDAYS: [&str; 7] = [
    "segunda", "terça", "quarta", "quinta", "sexta", "sábado", "domingo",
];
pub(super) const MONTHS: [&str; 12] = [
    "janeiro",
    "fevereiro",
    "março",
    "abril",
    "maio",
    "junho",
    "julho",
    "agosto",
    "setembro",
    "outubro",
    "novembro",
    "dezembro",
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
    match format {
        ClockFormat::Time => format!(
            "{} {:02}:{:02}",
            weekday(moment),
            moment.hour(),
            moment.minute()
        ),
        ClockFormat::Date => format!(
            "{}, {:02} de {} de {}",
            weekday(moment),
            moment.day(),
            month(moment),
            moment.year()
        ),
    }
}

fn weekday(moment: &NaiveDateTime) -> &'static str {
    WEEKDAYS[moment.weekday().num_days_from_monday() as usize]
}

fn month(moment: &NaiveDateTime) -> &'static str {
    MONTHS[moment.month0() as usize]
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
