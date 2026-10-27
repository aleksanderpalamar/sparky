use super::calendar::Calendar;
use chrono::{Datelike, NaiveDate};

#[test]
fn calendario_tem_seis_semanas_e_fevereiro_respeita_anos_bissextos() {
    let calendar = Calendar::new(NaiveDate::from_ymd_opt(2024, 2, 20).unwrap());
    assert_eq!(calendar.days().len(), 42);
    assert_eq!(calendar.days()[0].date.weekday().num_days_from_monday(), 0);
    assert!(
        calendar
            .days()
            .iter()
            .any(|day| day.current && day.date.day() == 29)
    );
    let calendar = Calendar::new(NaiveDate::from_ymd_opt(2026, 2, 20).unwrap());
    assert!(
        !calendar
            .days()
            .iter()
            .any(|day| day.current && day.date.day() == 29)
    );
}

#[test]
fn navegacao_cruza_ano_e_hoje_retorna_ao_mes_atual() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
    let mut calendar = Calendar::new(date);
    calendar.previous();
    assert_eq!(calendar.title(), "dezembro de 2025");
    calendar.next();
    calendar.next();
    assert_eq!(calendar.title(), "fevereiro de 2026");
    calendar.show_today();
    assert_eq!(calendar.title(), "janeiro de 2026");
    assert!(
        calendar
            .days()
            .iter()
            .any(|day| day.today && day.date == date)
    );
}

#[test]
fn virada_do_dia_atualiza_hoje_sem_desfazer_navegacao_do_usuario() {
    let mut calendar = Calendar::new(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
    calendar.previous();
    calendar.update_today(NaiveDate::from_ymd_opt(2027, 1, 1).unwrap());
    assert_eq!(calendar.title(), "novembro de 2026");
    calendar.show_today();
    assert_eq!(calendar.title(), "janeiro de 2027");
}
