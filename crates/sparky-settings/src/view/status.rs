use crate::app::state::Mode;
use crate::domain::section::Section;

const SECTIONS: &str = "1–9 seções";

pub fn hints(section: Section, mode: Mode) -> String {
    match mode {
        Mode::Editing(_) => "Enter salva · Esc cancela".to_owned(),
        Mode::Browse => format!("{} · {SECTIONS}", browsing(section)),
    }
}

fn browsing(section: Section) -> &'static str {
    match section {
        Section::Appearance | Section::Keyboard | Section::Apps => {
            "j/k escolhe · h/l ajusta · Enter edita · Tab troca de seção · q fecha"
        }
        Section::Autostart => {
            "j/k escolhe · Enter edita ou adiciona · d remove · Tab troca de seção"
        }
        Section::Shortcuts => "j/k rola · para mudar um atalho, escreva [[bind]] no sparky.toml",
        Section::Monitors => "F5 atualiza · Tab troca de seção · q fecha",
        Section::Network => "Edite os campos · Salvar e aplicar · Tab troca de seção",
        Section::Power => "j/k escolhe · h/l ajusta · F5 atualiza · Tab troca de seção",
        Section::DateTime => {
            "j/k escolhe · h/l alterna · Enter digita o fuso · F5 atualiza · Tab troca de seção"
        }
        Section::Region => {
            "j/k escolhe · h/l troca · Enter digita · F5 atualiza · Tab troca de seção"
        }
        Section::Users => "j/k escolhe conta · F5 atualiza · Tab troca de seção",
        Section::Media => "j/k escolhe volume · Enter alterna · h desmonta · l monta · F5 atualiza",
        Section::Themes => {
            "j/k escolhe · h/l troca o tema ou o papel de parede · Enter digita · Tab troca de seção"
        }
        Section::About => "Tab troca de seção · q fecha",
    }
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
