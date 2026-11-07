use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Greeting {
    Quiet,
    Fetch,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Terminal {
    pub fetch_on_start: bool,
}

impl Terminal {
    pub fn greeting(&self) -> Greeting {
        match self.fetch_on_start {
            true => Greeting::Fetch,
            false => Greeting::Quiet,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Greeting, Terminal};
    use crate::load::parse;
    use crate::schema::Config;

    #[test]
    fn por_padrao_o_terminal_abre_sem_o_fetch() {
        assert_eq!(
            Config::default().terminal,
            Terminal {
                fetch_on_start: false
            }
        );
        assert_eq!(Config::default().terminal.greeting(), Greeting::Quiet);
    }

    #[test]
    fn a_preferencia_liga_o_fetch_ao_abrir() {
        let Ok(config) = parse("[terminal]\nfetch_on_start = true\n") else {
            panic!("a seção terminal deveria ser aceita");
        };
        assert_eq!(config.terminal.greeting(), Greeting::Fetch);
    }

    #[test]
    fn chave_desconhecida_na_secao_e_recusada() {
        assert!(parse("[terminal]\nfetch = true\n").is_err());
    }
}
