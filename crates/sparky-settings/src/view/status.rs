use iced::alignment::Vertical;
use iced::widget::{container, row, text};
use iced::{Element, Length};
use sparky_ui::style::{self, PADDING, SMALL_TEXT, SPACING, STATUS_HEIGHT, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::{Mode, Settings};
use crate::domain::section::Section;

pub fn status(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let left = match &settings.notice {
        Some(notice) => text(notice.text().to_owned()).color(notice.color(skin)),
        None => text(""),
    };
    let line = row![
        left.size(TEXT_SIZE).width(Length::Fill),
        text(hints(settings.section, settings.mode)).size(SMALL_TEXT).color(skin.dark_foreground),
    ]
    .spacing(SPACING)
    .align_y(Vertical::Center);
    container(line)
        .height(STATUS_HEIGHT)
        .width(Length::Fill)
        .padding([0.0, PADDING])
        .align_y(Vertical::Center)
        .style(move |_| style::panel(skin))
        .into()
}

pub fn hints(section: Section, mode: Mode) -> &'static str {
    match (mode, section) {
        (Mode::Editing(_), _) => "Enter salva · Esc cancela",
        (Mode::Browse, Section::Appearance | Section::Keyboard | Section::Apps) => {
            "j/k escolhe · h/l ajusta · Enter edita · Tab troca de seção · q fecha"
        }
        (Mode::Browse, Section::Autostart) => "j/k escolhe · Enter edita ou adiciona · d remove · Tab troca de seção",
        (Mode::Browse, Section::Shortcuts) => "j/k rola · para mudar um atalho, escreva [[bind]] no sparky.toml",
        (Mode::Browse, Section::Monitors) => "F5 atualiza · Tab troca de seção · q fecha",
        (Mode::Browse, Section::Power) => "j/k escolhe · h/l ajusta · F5 atualiza · Tab troca de seção",
        (Mode::Browse, Section::DateTime) => "j/k escolhe · Enter edita · h/l alterna · F5 atualiza · Tab troca de seção",
        (Mode::Browse, Section::Region) => "j/k escolhe · Enter edita · h/l escolhe opção · F5 atualiza · Tab troca de seção",
        (Mode::Browse, Section::Users) => "j/k escolhe conta · F5 atualiza · Tab troca de seção",
        (Mode::Browse, Section::About) => "Tab troca de seção · q fecha",
    }
}

#[cfg(test)]
mod tests {
    use super::hints;
    use crate::app::state::{Mode, Slot, Target};
    use crate::domain::section::Section;

    #[test]
    fn editando_as_dicas_sao_de_salvar_ou_cancelar() {
        assert_eq!(hints(Section::Autostart, Mode::Editing(Target::Autostart(Slot::New))), "Enter salva · Esc cancela");
    }

    #[test]
    fn cada_secao_ensina_as_proprias_teclas() {
        assert!(hints(Section::Autostart, Mode::Browse).contains("d remove"));
        assert!(hints(Section::Keyboard, Mode::Browse).contains("h/l ajusta"));
    }
}
