pub const BAR: &str = "sparky-bar";
pub const NOTIFICATIONS: &str = "mako";
pub const POLKIT_AGENTS: [&str; 6] = [
    "/usr/lib/polkit-gnome/polkit-gnome-authentication-agent-1",
    "/usr/libexec/polkit-gnome-authentication-agent-1",
    "/usr/lib/policykit-1-gnome/polkit-gnome-authentication-agent-1",
    "/usr/libexec/polkit-mate-authentication-agent-1",
    "/usr/lib/mate-polkit/polkit-mate-authentication-agent-1",
    "lxpolkit",
];

pub fn components(installed: impl Fn(&str) -> bool) -> Vec<String> {
    let polkit = POLKIT_AGENTS.iter().copied().find(|agent| installed(agent));
    [Some(BAR), Some(NOTIFICATIONS), polkit]
        .into_iter()
        .flatten()
        .filter(|program| installed(program))
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{BAR, NOTIFICATIONS, components};

    #[test]
    fn sobe_a_barra_as_notificacoes_e_um_agente_polkit() {
        let instalados = [BAR, NOTIFICATIONS, "/usr/libexec/polkit-gnome-authentication-agent-1"];
        assert_eq!(
            components(|programa| instalados.contains(&programa)),
            vec!["sparky-bar", "mako", "/usr/libexec/polkit-gnome-authentication-agent-1"]
        );
    }

    #[test]
    fn so_o_primeiro_agente_polkit_instalado_sobe() {
        let found = components(|programa| programa.contains("polkit"));
        assert_eq!(found, vec!["/usr/lib/polkit-gnome/polkit-gnome-authentication-agent-1"]);
    }

    #[test]
    fn o_que_nao_esta_instalado_fica_de_fora() {
        assert!(components(|_| false).is_empty());
        assert_eq!(components(|programa| programa == BAR), vec!["sparky-bar"]);
    }
}
