use crate::domain::search::Order;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Apps,
    System,
}

impl Mode {
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Option<Self> {
        let args: Vec<String> = args.into_iter().collect();
        match args
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            [] => Some(Mode::Apps),
            ["--system"] => Some(Mode::System),
            _ => None,
        }
    }

    pub fn search(self) -> Option<String> {
        match self {
            Mode::Apps => Some(crate::i18n::text("search-app")),
            Mode::System => None,
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
            Mode::Apps => (800, 600),
            Mode::System => (440, 252),
        }
    }

    pub fn columns(self) -> usize {
        match self {
            Mode::Apps => 6,
            Mode::System => 1,
        }
    }

    pub fn rows(self) -> usize {
        match self {
            Mode::Apps => 4,
            Mode::System => 8,
        }
    }

    pub fn first_selection(self) -> Option<usize> {
        match self {
            Mode::Apps => None,
            Mode::System => Some(0),
        }
    }

    pub fn has_shelves(self) -> bool {
        matches!(self, Mode::Apps)
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
    fn o_menu_do_sistema_mantem_a_ordem_e_menor_e_nao_tem_busca() {
        assert_eq!(Mode::System.order(), Order::AsListed);
        assert_eq!(Mode::Apps.order(), Order::ByName);
        assert!(Mode::System.size().1 < Mode::Apps.size().1);
        assert_eq!(Mode::System.search(), None);
        assert_eq!(Mode::Apps.search().as_deref(), Some("Buscar aplicativo"));
    }

    #[test]
    fn aplicativos_aparecem_em_grade_com_prateleiras_e_o_menu_do_sistema_em_lista() {
        assert_eq!((Mode::Apps.columns(), Mode::Apps.rows()), (6, 4));
        assert_eq!(Mode::System.columns(), 1);
        assert!(Mode::Apps.has_shelves());
        assert!(!Mode::System.has_shelves());
        assert_eq!(Mode::Apps.first_selection(), None);
        assert_eq!(Mode::System.first_selection(), Some(0));
    }
}
