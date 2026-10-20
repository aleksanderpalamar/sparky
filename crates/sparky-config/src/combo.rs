#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mods {
    pub logo: bool,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySpec {
    Code(u32),
    Name(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Combo {
    pub mods: Mods,
    pub key: KeySpec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComboError {
    Empty,
    UnknownModifier(String),
    InvalidCode(String),
}

impl Combo {
    pub fn parse(text: &str) -> Result<Self, ComboError> {
        let mut parts: Vec<&str> = text.split('+').map(str::trim).collect();
        let key = parts.pop().filter(|key| !key.is_empty()).ok_or(ComboError::Empty)?;
        let mut mods = Mods::default();
        for part in parts {
            apply_modifier(&mut mods, part)?;
        }
        Ok(Self {
            mods,
            key: key_spec(key)?,
        })
    }
}

pub fn same_keys(first: &str, second: &str) -> bool {
    match (Combo::parse(first), Combo::parse(second)) {
        (Ok(first), Ok(second)) => first.mods == second.mods && same_key(&first.key, &second.key),
        _ => first == second,
    }
}

fn same_key(first: &KeySpec, second: &KeySpec) -> bool {
    match (first, second) {
        (KeySpec::Name(first), KeySpec::Name(second)) => first.eq_ignore_ascii_case(second),
        _ => first == second,
    }
}

fn apply_modifier(mods: &mut Mods, name: &str) -> Result<(), ComboError> {
    match name.to_ascii_uppercase().as_str() {
        "SUPER" | "LOGO" | "WIN" => mods.logo = true,
        "SHIFT" => mods.shift = true,
        "CTRL" | "CONTROL" => mods.ctrl = true,
        "ALT" => mods.alt = true,
        _ => return Err(ComboError::UnknownModifier(name.to_owned())),
    }
    Ok(())
}

fn key_spec(key: &str) -> Result<KeySpec, ComboError> {
    let Some(code) = key.strip_prefix("code:") else {
        return Ok(KeySpec::Name(key.to_owned()));
    };
    code.parse()
        .map(KeySpec::Code)
        .map_err(|_| ComboError::InvalidCode(key.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::{Combo, ComboError, KeySpec, Mods, same_keys};

    #[test]
    fn mesma_combinacao_escrita_de_outro_jeito() {
        assert!(same_keys("SUPER + SHIFT + f", "shift + super + F"));
        assert!(same_keys("SUPER + code:10", "LOGO + code:10"));
        assert!(!same_keys("SUPER + F", "SUPER + SHIFT + F"));
        assert!(!same_keys("SUPER + code:10", "SUPER + code:11"));
    }

    #[test]
    fn le_modificadores_e_tecla() {
        let combo = Combo::parse("SUPER + SHIFT + RETURN");
        assert_eq!(
            combo,
            Ok(Combo {
                mods: Mods {
                    logo: true,
                    shift: true,
                    ..Mods::default()
                },
                key: KeySpec::Name("RETURN".to_owned()),
            })
        );
    }

    #[test]
    fn tecla_por_codigo_independe_do_layout() {
        let Ok(combo) = Combo::parse("SUPER + code:10") else {
            panic!("deveria ler o código");
        };
        assert_eq!(combo.key, KeySpec::Code(10));
    }

    #[test]
    fn tecla_sozinha_nao_tem_modificador() {
        let Ok(combo) = Combo::parse("PRINT") else {
            panic!("deveria ler a tecla");
        };
        assert_eq!(combo.mods, Mods::default());
    }

    #[test]
    fn aceita_ctrl_e_control() {
        let Ok(combo) = Combo::parse("control + alt + TAB") else {
            panic!("deveria ler");
        };
        assert!(combo.mods.ctrl && combo.mods.alt);
    }

    #[test]
    fn recusa_modificador_desconhecido() {
        assert_eq!(
            Combo::parse("HYPER + A"),
            Err(ComboError::UnknownModifier("HYPER".to_owned()))
        );
    }

    #[test]
    fn recusa_combinacao_vazia_e_codigo_invalido() {
        assert_eq!(Combo::parse("SUPER + "), Err(ComboError::Empty));
        assert_eq!(Combo::parse("code:x"), Err(ComboError::InvalidCode("code:x".to_owned())));
    }
}
