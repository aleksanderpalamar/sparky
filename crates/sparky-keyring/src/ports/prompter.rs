use zeroize::Zeroizing;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ask {
    Create,
    Unlock,
    Retry,
}

pub enum Answer {
    Password(Zeroizing<String>),
    Cancelled,
}

pub trait Prompter: Send + Sync {
    fn ask(&self, ask: Ask) -> Answer;
}
