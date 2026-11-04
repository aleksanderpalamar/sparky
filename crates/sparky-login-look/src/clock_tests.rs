use chrono::{NaiveDate, NaiveTime};

use super::clock::{date, time};

fn hora(hour: u32, minute: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(hour, minute, 0).unwrap_or_default()
}

fn dia(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or_default()
}

#[test]
fn a_hora_sai_em_doze_horas_com_am_e_pm_como_na_foto() {
    assert_eq!(time(hora(18, 19)), "06:19 PM");
    assert_eq!(time(hora(6, 5)), "06:05 AM");
}

#[test]
fn meia_noite_e_meio_dia_sao_doze() {
    assert_eq!(time(hora(0, 0)), "12:00 AM");
    assert_eq!(time(hora(12, 30)), "12:30 PM");
}

#[test]
fn a_data_traz_o_dia_da_semana_o_dia_e_o_mes_por_extenso() {
    let plain = |value: String| value.replace(['\u{2068}', '\u{2069}'], "");
    assert_eq!(
        plain(date(dia(2026, 11, 2))),
        "segunda-feira, 2 de novembro"
    );
    assert_eq!(plain(date(dia(2027, 1, 15))), "sexta-feira, 15 de janeiro");
}
