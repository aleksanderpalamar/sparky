use crate::app::state::Mode;
use crate::domain::section::Section;

pub fn hints(section: Section, mode: Mode) -> String {
    match mode {
        Mode::Editing(_) => crate::i18n::text("hint-editing"),
        Mode::Browse => format!(
            "{} · {}",
            browsing(section),
            crate::i18n::text("hint-sections")
        ),
    }
}

fn browsing(section: Section) -> String {
    let key = match section {
        Section::Appearance | Section::Keyboard | Section::Apps => "hint-fields",
        Section::Autostart => "hint-autostart",
        Section::Shortcuts => "hint-shortcuts",
        Section::Monitors => "hint-monitors",
        Section::Network => "hint-network",
        Section::Power => "hint-power",
        Section::DateTime => "hint-date-time",
        Section::Region => "hint-region",
        Section::Users => "hint-users",
        Section::Media => "hint-media",
        Section::Themes => "hint-themes",
        Section::About => "hint-about",
    };
    crate::i18n::text(key)
}

#[cfg(test)]
mod tests {
    use super::hints;
    use crate::app::state::{Mode, Slot, Target};
    use crate::domain::section::Section;

    #[test]
    fn editando_as_dicas_sao_de_salvar_ou_cancelar() {
        assert_eq!(
            hints(
                Section::Autostart,
                Mode::Editing(Target::Autostart(Slot::New))
            ),
            "Enter salva · Esc cancela"
        );
    }

    #[test]
    fn cada_secao_ensina_as_proprias_teclas() {
        assert!(hints(Section::Autostart, Mode::Browse).contains("d remove"));
        assert!(hints(Section::Keyboard, Mode::Browse).contains("h/l ajusta"));
    }
}
