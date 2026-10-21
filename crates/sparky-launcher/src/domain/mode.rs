use crate::domain::search::Order;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Apps,
    System,
}

impl Mode {
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Option<Self> {
        let args: Vec<String> = args.into_iter().collect();
        match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
            [] => Some(Mode::Apps),
            ["--system"] => Some(Mode::System),
            _ => None,
        }
    }

    pub fn placeholder(self) -> &'static str {
        match self {
            Mode::Apps => "Buscar aplicativo",
            Mode::System => "Sistema",
        }
    }

    pub fn order(self) -> Order {
        match self {
            Mode::Apps => Order::ByName,
            Mode::System => Order::AsListed,
        }
    }

    pub fn size(self) -> (u32, u32) {
        match self {
            Mode::Apps => (640, 440),
            Mode::System => (440, 310),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Mode;
    use crate::domain::search::Order;

    fn modo(args: &[&str]) -> Option<Mode> {
        Mode::from_args(args.iter().map(|arg| arg.to_string()))
    }

    #[test]
    fn sem_argumento_lista_aplicativos_e_system_abre_o_menu_do_sistema() {
        assert_eq!(modo(&[]), Some(Mode::Apps));
        assert_eq!(modo(&["--system"]), Some(Mode::System));
        assert_eq!(modo(&["--voar"]), None);
    }

    #[test]
    fn o_menu_do_sistema_mantem_a_ordem_e_e_menor() {
        assert_eq!(Mode::System.order(), Order::AsListed);
        assert_eq!(Mode::Apps.order(), Order::ByName);
        assert!(Mode::System.size().1 < Mode::Apps.size().1);
        assert_eq!(Mode::System.placeholder(), "Sistema");
    }
}
