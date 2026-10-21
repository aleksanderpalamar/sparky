#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Section {
    #[default]
    Appearance,
    Keyboard,
    Apps,
    Autostart,
    Shortcuts,
    Monitors,
    Power,
    DateTime,
    Region,
    Users,
    Media,
    About,
}

impl Section {
    pub const ALL: [Section; 12] = [
        Section::Appearance,
        Section::Keyboard,
        Section::Apps,
        Section::Autostart,
        Section::Shortcuts,
        Section::Monitors,
        Section::Power,
        Section::DateTime,
        Section::Region,
        Section::Users,
        Section::Media,
        Section::About,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Section::Appearance => "Aparência",
            Section::Keyboard => "Teclado",
            Section::Apps => "Aplicativos",
            Section::Autostart => "Inicialização",
            Section::Shortcuts => "Atalhos",
            Section::Monitors => "Monitores",
            Section::Power => "Energia",
            Section::DateTime => "Data e hora",
            Section::Region => "Região e idioma",
            Section::Users => "Usuários",
            Section::Media => "Mídia",
            Section::About => "Sobre",
        }
    }

    pub fn summary(self) -> &'static str {
        match self {
            Section::Appearance => "Folgas, borda e tamanho das janelas flutuantes",
            Section::Keyboard => "Layout e repetição das teclas",
            Section::Apps => "Programas que os atalhos do sparky abrem",
            Section::Autostart => "Comandos rodados quando a sessão começa",
            Section::Shortcuts => "Atalhos em uso: os padrões e os do sparky.toml",
            Section::Monitors => "Monitores que o sparky está usando",
            Section::Power => "Perfil de energia, brilho da tela e bateria",
            Section::DateTime => "Fuso horário, sincronização pela rede e relógio do hardware",
            Section::Region => "Idioma e teclados padrão do sistema",
            Section::Users => "Contas, grupos e sessões abertas",
            Section::Media => "Montagem e desmontagem de volumes externos",
            Section::About => "O sistema e o sparky",
        }
    }

    pub fn index(self) -> usize {
        Section::ALL.iter().position(|section| *section == self).unwrap_or(0)
    }

    pub fn nth(index: usize) -> Option<Self> {
        Section::ALL.get(index).copied()
    }

    pub fn next(self) -> Self {
        Section::ALL[(self.index() + 1) % Section::ALL.len()]
    }

    pub fn previous(self) -> Self {
        Section::ALL[(self.index() + Section::ALL.len() - 1) % Section::ALL.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::Section;

    #[test]
    fn tab_percorre_as_secoes_e_volta_ao_comeco() {
        assert_eq!(Section::Appearance.next(), Section::Keyboard);
        assert_eq!(Section::About.next(), Section::Appearance);
        assert_eq!(Section::Appearance.previous(), Section::About);
    }

    #[test]
    fn os_numeros_levam_a_cada_secao() {
        assert_eq!(Section::nth(0), Some(Section::Appearance));
        assert_eq!(Section::nth(6), Some(Section::Power));
        assert_eq!(Section::nth(7), Some(Section::DateTime));
        assert_eq!(Section::nth(8), Some(Section::Region));
        assert_eq!(Section::nth(9), Some(Section::Users));
        assert_eq!(Section::nth(10), Some(Section::Media));
        assert_eq!(Section::nth(11), Some(Section::About));
        assert_eq!(Section::nth(12), None);
        assert_eq!(Section::Monitors.index(), 5);
    }
}
