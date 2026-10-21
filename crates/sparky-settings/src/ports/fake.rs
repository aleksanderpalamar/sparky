use std::cell::RefCell;
use std::collections::HashMap;

use crate::ports::commands::{CommandError, CommandRunner, invocation};

#[derive(Debug, Default)]
pub struct FakeCommands {
    replies: HashMap<String, Result<String, CommandError>>,
    calls: RefCell<Vec<String>>,
}

impl FakeCommands {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, call: &str, output: &str) -> Self {
        self.replies.insert(call.to_owned(), Ok(output.to_owned()));
        self
    }

    pub fn accepting(self, call: &str) -> Self {
        self.with(call, "")
    }

    pub fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl CommandRunner for FakeCommands {
    fn run(&self, program: &str, arguments: &[&str]) -> Result<String, CommandError> {
        let call = invocation(program, arguments);
        self.calls.borrow_mut().push(call.clone());
        self.replies.get(&call).cloned().unwrap_or(Err(CommandError::Unavailable))
    }
}
