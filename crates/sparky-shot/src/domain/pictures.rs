use chrono::NaiveDateTime;

pub const FOLDER: &str = "Capturas de tela";

pub fn file_name(moment: &NaiveDateTime) -> String {
    format!("captura-{}.png", moment.format("%Y-%m-%d_%H-%M-%S"))
}

#[cfg(test)]
mod tests {
    use super::file_name;
    use chrono::NaiveDate;

    #[test]
    fn o_nome_leva_data_e_hora_e_ordena_bem() {
        let Some(momento) = NaiveDate::from_ymd_opt(2026, 10, 7).and_then(|d| d.and_hms_opt(9, 5, 3)) else {
            panic!("momento inválido");
        };
        assert_eq!(file_name(&momento), "captura-2026-10-07_09-05-03.png");
    }
}
