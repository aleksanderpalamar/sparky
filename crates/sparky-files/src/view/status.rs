use iced::alignment::Vertical;
use iced::widget::{container, row, text, text_input};
use iced::{Element, Length};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::domain::clip::Clip;
use crate::domain::entry::Kind;
use crate::domain::wording::{items, summary};
use sparky_ui::prompt::PROMPT;
use sparky_ui::style::{self, PADDING, SMALL_TEXT, SPACING, STATUS_HEIGHT, TEXT_SIZE};

const HINTS: &str = "Enter abrir · h voltar · / filtrar · Ctrl+1 grade · Ctrl+2 lista";
const PROMPT_WIDTH: f32 = 420.0;

pub fn status(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let content = match &files.mode {
        Mode::Browse => browsing(files),
        Mode::Filter => prompt(files, "Filtrar", "Enter fixa · Esc limpa"),
        Mode::Rename(_) => prompt(files, "Renomear", "Enter confirma · Esc cancela"),
        Mode::Create => prompt(
            files,
            "Criar (termine com / para pasta)",
            "Enter cria · Esc cancela",
        ),
        Mode::New(Kind::Directory) => prompt(files, "Nova pasta", "Enter cria · Esc cancela"),
        Mode::New(Kind::File) => prompt(files, "Novo documento", "Enter cria · Esc cancela"),
        Mode::Destination(clip) => prompt(
            files,
            destination(clip),
            "Caminho da pasta · Enter confirma · Esc cancela",
        ),
        Mode::Compress(_) => prompt(files, "Comprimir (.tar.gz)", "Enter cria · Esc cancela"),
        Mode::ConfirmTrash(names) => text(question(names))
            .size(TEXT_SIZE)
            .color(skin.urgent)
            .into(),
        Mode::ConfirmEmptyTrash(count) => text(format!(
            "Excluir permanentemente {count} itens da lixeira? Enter confirma · Esc cancela"
        ))
        .size(TEXT_SIZE)
        .color(skin.urgent)
        .into(),
    };
    container(content)
        .height(STATUS_HEIGHT)
        .width(Length::Fill)
        .padding([0.0, PADDING])
        .align_y(Vertical::Center)
        .style(move |_| style::panel(skin))
        .into()
}

fn destination(clip: &Clip) -> &'static str {
    match clip {
        Clip::Cut(_) => "Mover para",
        Clip::Copy(_) => "Copiar para",
    }
}

pub fn question(names: &[String]) -> String {
    let target = match names {
        [name] => name.clone(),
        many => items(many.len()),
    };
    format!("Mover {target} para a lixeira? Enter confirma · Esc cancela")
}

fn browsing(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let left = match &files.notice {
        Some(notice) => text(notice.text().to_owned()).color(notice.color(skin)),
        None => text(summary(
            files.listing.len(),
            files.listing.marked_count(),
            files.clip.as_ref(),
        ))
        .color(skin.foreground),
    };
    let hints = match files.size.width < 720.0 {
        true => "Ctrl+1 grade · Ctrl+2 lista",
        false => HINTS,
    };
    row![left.size(TEXT_SIZE).width(Length::Fill), hint(skin, hints)]
        .spacing(SPACING)
        .align_y(Vertical::Center)
        .into()
}

fn prompt<'a>(files: &'a Files, label: &'static str, keys: &'static str) -> Element<'a, Message> {
    let skin = files.skin;
    let input = text_input("", &files.input)
        .id(PROMPT)
        .on_input(Message::Typed)
        .on_submit(Message::Submit)
        .size(TEXT_SIZE)
        .width(PROMPT_WIDTH)
        .style(style::input);
    row![
        text(label).size(TEXT_SIZE).color(skin.accent),
        input,
        hint(skin, keys)
    ]
    .spacing(SPACING)
    .align_y(Vertical::Center)
    .into()
}

fn hint(skin: Skin, keys: &'static str) -> Element<'static, Message> {
    text(keys)
        .size(SMALL_TEXT)
        .color(skin.dark_foreground)
        .into()
}

#[cfg(test)]
mod tests {
    use super::question;

    #[test]
    fn a_pergunta_cita_o_nome_de_um_item_ou_a_contagem() {
        assert_eq!(
            question(&["a.txt".to_owned()]),
            "Mover a.txt para a lixeira? Enter confirma · Esc cancela"
        );
        assert_eq!(
            question(&["a".to_owned(), "b".to_owned()]),
            "Mover 2 itens para a lixeira? Enter confirma · Esc cancela"
        );
    }
}
