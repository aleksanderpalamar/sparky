const UNITS: [&str; 4] = ["kB", "MB", "GB", "TB"];
const STEP: f64 = 1000.0;

pub fn human(bytes: u64) -> String {
    if bytes < 1000 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64 / STEP;
    let mut unit = 0;
    while value >= STEP && unit + 1 < UNITS.len() {
        value /= STEP;
        unit += 1;
    }
    let number = match value < 10.0 {
        true => format!("{value:.1}").replace('.', ","),
        false => format!("{value:.0}"),
    };
    format!("{number} {}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::human;

    #[test]
    fn bytes_ficam_inteiros() {
        assert_eq!(human(0), "0 B");
        assert_eq!(human(999), "999 B");
    }

    #[test]
    fn abaixo_de_dez_tem_uma_casa_com_virgula() {
        assert_eq!(human(1000), "1,0 kB");
        assert_eq!(human(1536), "1,5 kB");
        assert_eq!(human(5_300_000), "5,3 MB");
    }

    #[test]
    fn de_dez_para_cima_fica_inteiro() {
        assert_eq!(human(12_345), "12 kB");
        assert_eq!(human(250_000_000_000), "250 GB");
    }

    #[test]
    fn para_no_terabyte() {
        assert_eq!(human(3_000_000_000_000_000), "3000 TB");
    }
}
