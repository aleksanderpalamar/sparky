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
    Themes,
    Network,
    About,
}

impl Section {
    pub const ALL: [Section; 14] = [
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
        Section::Themes,
        Section::Network,
        Section::About,
    ];

    pub fn label(self) -> String {
        crate::i18n::text(&format!("section-{}-label", self.key()))
    }

    pub fn summary(self) -> String {
        crate::i18n::text(&format!("section-{}-summary", self.key()))
    }

    fn key(self) -> &'static str {
        match self {
            Section::Appearance => "appearance",
            Section::Keyboard => "keyboard",
            Section::Apps => "apps",
            Section::Autostart => "autostart",
            Section::Shortcuts => "shortcuts",
            Section::Monitors => "monitors",
            Section::Power => "power",
            Section::DateTime => "date-time",
            Section::Region => "region",
            Section::Users => "users",
            Section::Media => "media",
            Section::Themes => "themes",
            Section::Network => "network",
            Section::About => "about",
        }
    }

    pub fn index(self) -> usize {
        Section::ALL
            .iter()
            .position(|section| *section == self)
            .unwrap_or(0)
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
        assert_eq!(Section::nth(11), Some(Section::Themes));
        assert_eq!(Section::nth(12), Some(Section::Network));
        assert_eq!(Section::nth(13), Some(Section::About));
        assert_eq!(Section::nth(14), None);
        assert_eq!(Section::Monitors.index(), 5);
    }
}
