use crate::action::Bind;
use crate::combo::same_keys;
use crate::defaults;
use crate::schema::Config;

pub fn merged(config: &Config, terminal: &str) -> Vec<Bind> {
    let mut binds = match config.general.default_binds {
        true => defaults::sparky(&config.apps, terminal),
        false => Vec::new(),
    };
    for custom in &config.bind {
        binds.retain(|known| !same_keys(&known.keys, &custom.keys));
        binds.push(custom.clone());
    }
    binds
}

#[cfg(test)]
mod tests {
    use super::merged;
    use crate::action::{Action, Bind};
    use crate::schema::{Config, General};

    fn acao(binds: &[Bind], keys: &str) -> Option<Action> {
        binds.iter().find(|bind| bind.keys == keys).map(|bind| bind.action.clone())
    }

    #[test]
    fn sem_atalho_do_usuario_valem_os_padroes_com_o_terminal_escolhido() {
        let binds = merged(&Config::default(), "foot");
        assert_eq!(acao(&binds, "SUPER + RETURN"), Some(Action::Spawn { command: "foot".to_owned() }));
    }

    #[test]
    fn o_atalho_do_usuario_substitui_o_padrao_da_mesma_combinacao() {
        let config = Config {
            bind: vec![Bind::new("super + return", Action::Spawn { command: "alacritty".to_owned() })],
            ..Config::default()
        };
        let binds = merged(&config, "kitty");
        assert_eq!(acao(&binds, "SUPER + RETURN"), None);
        assert_eq!(acao(&binds, "super + return"), Some(Action::Spawn { command: "alacritty".to_owned() }));
    }

    #[test]
    fn atalho_novo_se_soma_aos_padroes() {
        let antes = merged(&Config::default(), "kitty").len();
        let config = Config {
            bind: vec![Bind::new("SUPER + B", Action::Spawn { command: "firefox".to_owned() })],
            ..Config::default()
        };
        assert_eq!(merged(&config, "kitty").len(), antes + 1);
    }

    #[test]
    fn sem_os_padroes_ficam_so_os_do_usuario() {
        let config = Config {
            general: General {
                default_binds: false,
                ..General::SPARKY
            },
            bind: vec![Bind::new("SUPER + Q", Action::CloseWindow)],
            ..Config::default()
        };
        assert_eq!(merged(&config, "kitty"), config.bind);
    }
}
