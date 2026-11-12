use iced::alignment::Vertical;
use iced::widget::{container, row, text, text_input};
use iced::{Element, Length};
use sparky_theme::Skin;

use crate::app::jobs::{Job, Pending};
use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::domain::clip::Clip;
use crate::domain::entry::Kind;
use crate::domain::removal::Removal;
use crate::domain::transfer::Action;
use crate::domain::wording::summary;
use crate::i18n;
use fluent_bundle::FluentArgs;
use sparky_ui::notice::Notice;
use sparky_ui::prompt::PROMPT;
use sparky_ui::style::{self, PADDING, SMALL_TEXT, SPACING, STATUS_HEIGHT, TEXT_SIZE};

const PROMPT_WIDTH: f32 = 420.0;

pub fn status(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let content = match &files.mode {
        Mode::Browse => browsing(files),
        Mode::Filter => prompt(files, i18n::text("filter"), i18n::text("filter-hint")),
        Mode::Rename(_) => prompt(files, i18n::text("rename"), i18n::text("confirm-hint")),
        Mode::Create => prompt(files, i18n::text("create-hint"), i18n::text("create-keys")),
        Mode::New(Kind::Directory) => {
            prompt(files, i18n::text("new-folder"), i18n::text("create-keys"))
        }
        Mode::New(Kind::File) => prompt(files, i18n::text("new-file"), i18n::text("create-keys")),
        Mode::Destination(clip) => prompt(files, destination(clip), i18n::text("destination-hint")),
        Mode::Compress(_) => prompt(files, i18n::text("compress"), i18n::text("create-keys")),
        Mode::ConfirmRemoval(removal, names) => text(question(*removal, names))
            .size(TEXT_SIZE)
            .color(skin.urgent)
            .into(),
        Mode::Conflict(pending) => text(conflict(pending))
            .size(TEXT_SIZE)
            .color(skin.urgent)
            .into(),
        Mode::ConfirmEmptyTrash(count) => text(i18n::with(
            "confirm-empty-trash",
            "count",
            count.to_string(),
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

fn destination(clip: &Clip) -> String {
    match clip {
        Clip::Cut(_) => i18n::text("move-to"),
        Clip::Copy(_) => i18n::text("copy-to"),
    }
}

pub fn conflict(pending: &Pending) -> String {
    let mut args = FluentArgs::new();
    args.set("count", pending.names.len());
    args.set("name", pending.names.first().cloned().unwrap_or_default());
    i18n::message("conflict-question", Some(&args))
}

pub fn progress(job: &Job) -> String {
    let key = match job.action {
        Action::Copy => "job-copying",
        Action::Move => "job-moving",
    };
    let mut args = FluentArgs::new();
    args.set("done", job.progress.items_done);
    args.set("total", job.progress.items_total);
    args.set("percent", job.progress.percent());
    i18n::message(key, Some(&args))
}

pub fn question(removal: Removal, names: &[String]) -> String {
    let (one, many) = match removal {
        Removal::Trash => ("confirm-trash-one", "confirm-trash-many"),
        Removal::Delete => ("confirm-delete-one", "confirm-delete-many"),
    };
    match names {
        [name] => i18n::with(one, "name", name.clone()),
        names => i18n::with(many, "count", names.len().to_string()),
    }
}

fn browsing(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let left = match (&files.job, &files.notice) {
        (Some(job), _) => text(progress(job)).color(skin.accent),
        (None, notice) => browsing_text(files, notice.as_ref()),
    };
    let hints = match files.size.width < 720.0 {
        true => "short-hints",
        false => "browse-hints",
    };
    row![
        left.size(TEXT_SIZE).width(Length::Fill),
        hint(skin, i18n::text(hints))
    ]
    .spacing(SPACING)
    .align_y(Vertical::Center)
    .into()
}

fn browsing_text<'a>(files: &Files, notice: Option<&Notice>) -> iced::widget::Text<'a> {
    let skin = files.skin;
    match notice {
        Some(notice) => text(notice.text().to_owned()).color(notice.color(skin)),
        None => text(summary(
            files.listing.len(),
            files.listing.marked_count(),
            files.clip.as_ref(),
        ))
        .color(skin.foreground),
    }
}

fn prompt<'a>(files: &'a Files, label: String, keys: String) -> Element<'a, Message> {
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

fn hint(skin: Skin, keys: String) -> Element<'static, Message> {
    text(keys)
        .size(SMALL_TEXT)
        .color(skin.dark_foreground)
        .into()
}

#[cfg(test)]
mod tests {
    use super::question;
    use crate::domain::removal::Removal;

    #[test]
    fn a_pergunta_cita_o_nome_de_um_item_ou_a_contagem() {
        assert_eq!(
            question(Removal::Trash, &["a.txt".to_owned()]),
            "Mover a.txt para a lixeira? Enter confirma · Esc cancela"
        );
        assert_eq!(
            question(Removal::Trash, &["a".to_owned(), "b".to_owned()]),
            "Mover 2 itens para a lixeira? Enter confirma · Esc cancela"
        );
        assert_eq!(
            question(Removal::Delete, &["a.txt".to_owned()]),
            "Excluir a.txt permanentemente? Não pode ser desfeito. Enter confirma · Esc cancela"
        );
    }
}
