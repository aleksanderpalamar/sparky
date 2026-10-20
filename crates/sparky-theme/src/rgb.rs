#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub fn parse(spec: &str) -> Option<Self> {
        let trimmed = spec.trim();
        let digits = trimmed.strip_prefix('#').unwrap_or(trimmed);
        match digits.len() {
            3 => Self::from_short(digits),
            6 => Self::from_long(digits),
            _ => None,
        }
    }

    fn from_short(digits: &str) -> Option<Self> {
        let mut channels = digits.chars().map(|c| {
            let value = c.to_digit(16)? as u8;
            Some(value * 16 + value)
        });
        Self::collect(&mut channels)
    }

    fn from_long(digits: &str) -> Option<Self> {
        let mut channels = digits
            .as_bytes()
            .chunks(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok());
        Self::collect(&mut channels)
    }

    fn collect(channels: &mut impl Iterator<Item = Option<u8>>) -> Option<Self> {
        let red = channels.next()??;
        let green = channels.next()??;
        let blue = channels.next()??;
        Some(Self::new(red, green, blue))
    }
}

#[cfg(test)]
mod tests {
    use super::Rgb;

    #[test]
    fn le_hexadecimal_de_seis_digitos_com_cerquilha() {
        assert_eq!(Rgb::parse("#7aa2f7"), Some(Rgb::new(0x7a, 0xa2, 0xf7)));
    }

    #[test]
    fn le_hexadecimal_sem_cerquilha() {
        assert_eq!(Rgb::parse("1a1b26"), Some(Rgb::new(0x1a, 0x1b, 0x26)));
    }

    #[test]
    fn le_hexadecimal_de_tres_digitos_duplicando_cada_canal() {
        assert_eq!(Rgb::parse("#f0a"), Some(Rgb::new(0xff, 0x00, 0xaa)));
    }

    #[test]
    fn aceita_maiusculas_e_espacos_em_volta() {
        assert_eq!(Rgb::parse("  #7AA2F7 "), Some(Rgb::new(0x7a, 0xa2, 0xf7)));
    }

    #[test]
    fn recusa_comprimento_invalido() {
        assert_eq!(Rgb::parse("#7aa2f"), None);
        assert_eq!(Rgb::parse(""), None);
    }

    #[test]
    fn recusa_digito_nao_hexadecimal() {
        assert_eq!(Rgb::parse("#7aa2fz"), None);
        assert_eq!(Rgb::parse("#zza"), None);
    }
}
