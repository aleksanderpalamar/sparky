pub const KEYRING: &str = "sparky-keyring";
pub const POLKIT: &str = "polkit-sparky";
pub const BAR: &str = "sparky-bar";
pub const NOTIFICATIONS: &str = "mako";

pub fn components(installed: impl Fn(&str) -> bool) -> Vec<String> {
    [KEYRING, POLKIT, BAR, NOTIFICATIONS]
        .into_iter()
        .filter(|program| installed(program))
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{BAR, KEYRING, NOTIFICATIONS, POLKIT, components};

    #[test]
    fn o_chaveiro_e_o_agente_proprio_sobem_antes_dos_aplicativos() {
        let installed = [KEYRING, POLKIT, BAR, NOTIFICATIONS];
        assert_eq!(
            components(|program| installed.contains(&program)),
            installed
        );
    }

    #[test]
    fn agentes_externos_nao_sao_iniciados() {
        let installed = ["polkit-gnome", "lxpolkit", "polkit-mate"];
        assert!(components(|program| installed.contains(&program)).is_empty());
    }

    #[test]
    fn o_que_nao_esta_instalado_fica_de_fora() {
        assert!(components(|_| false).is_empty());
        assert_eq!(components(|program| program == BAR), vec![BAR]);
        assert_eq!(components(|program| program == POLKIT), vec![POLKIT]);
    }
}
