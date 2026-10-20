use iced::Task;
use iced::widget::operation;

pub const PROMPT: &str = "prompt";

pub fn focus<T: Send + 'static>() -> Task<T> {
    operation::focus(PROMPT).chain(operation::move_cursor_to_end(PROMPT))
}

pub fn select_stem<T: Send + 'static>(stem: usize) -> Task<T> {
    operation::focus(PROMPT).chain(operation::select_range(PROMPT, 0, stem))
}
