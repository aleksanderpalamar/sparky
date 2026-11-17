use std::io::Write;

use crate::domain::choice::Outcome;
use crate::domain::choice::wire::encode_outcome;
use crate::ports::chooser::Answer;

pub struct StdoutAnswer;

impl Answer for StdoutAnswer {
    fn send(&self, outcome: &Outcome) {
        let mut output = std::io::stdout().lock();
        let _ = output.write_all(encode_outcome(outcome).as_bytes());
        let _ = output.flush();
    }
}

pub fn mime_of(name: &str) -> String {
    gio::content_type_guess(Some(name), None).0.to_string()
}
