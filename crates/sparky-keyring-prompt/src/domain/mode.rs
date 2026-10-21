#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Create,
    Unlock,
    Retry,
}

impl Mode {
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Option<Self> {
        let args: Vec<String> = args.into_iter().collect();
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        match args.as_slice() {
            ["create"] => Some(Mode::Create),
            ["unlock"] => Some(Mode::Unlock),
            ["unlock", "--retry"] => Some(Mode::Retry),
            _ => None,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Mode::Create => "Crie a senha do cofre",
            Mode::Unlock | Mode::Retry => "Destrancar o cofre",
        }
    }

    pub fn explanation(self) -> &'static str {
        match self {
            Mode::Create => "Um aplicativo quer guardar uma senha. Escolha a senha que vai proteger o cofre do sparky.",
            Mode::Unlock | Mode::Retry => "Um aplicativo quer usar uma senha guardada. Digite a senha do cofre do sparky.",
        }
    }

    pub fn warning(self) -> Option<&'static str> {
        match self {
            Mode::Retry => Some("Senha incorreta. Tente de novo."),
            Mode::Create | Mode::Unlock => None,
        }
    }

    pub fn confirms(self) -> bool {
        self == Mode::Create
    }
}

#[cfg(test)]
mod tests {
    use super::Mode;

    fn modo(args: &[&str]) -> Option<Mode> {
        Mode::from_args(args.iter().map(|arg| arg.to_string()))
    }

    #[test]
    fn o_modo_vem_dos_argumentos() {
        assert_eq!(modo(&["create"]), Some(Mode::Create));
        assert_eq!(modo(&["unlock"]), Some(Mode::Unlock));
        assert_eq!(modo(&["unlock", "--retry"]), Some(Mode::Retry));
        assert_eq!(modo(&[]), None);
        assert_eq!(modo(&["create", "--retry"]), None);
    }

    #[test]
    fn so_a_nova_tentativa_avisa_da_senha_errada() {
        assert_eq!(Mode::Retry.warning(), Some("Senha incorreta. Tente de novo."));
        assert_eq!(Mode::Unlock.warning(), None);
        assert!(Mode::Create.confirms());
        assert!(!Mode::Retry.confirms());
    }
}
