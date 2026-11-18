pub const VERB: &str = "nice";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    Usage,
    OutOfRange,
    Protected,
    NotOwner,
    Gone,
    Failed,
}

const CODES: [(Refusal, u8); 6] = [
    (Refusal::Usage, 64),
    (Refusal::OutOfRange, 65),
    (Refusal::Protected, 66),
    (Refusal::NotOwner, 67),
    (Refusal::Gone, 68),
    (Refusal::Failed, 69),
];

impl Refusal {
    pub fn code(self) -> u8 {
        CODES
            .iter()
            .find(|(refusal, _)| *refusal == self)
            .map_or(1, |(_, code)| *code)
    }

    pub fn from_code(code: i32) -> Option<Self> {
        CODES
            .iter()
            .find(|(_, known)| i32::from(*known) == code)
            .map(|(refusal, _)| *refusal)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Renice {
    pub task: u32,
    pub started: u64,
    pub value: i8,
}

impl Renice {
    pub fn arguments(&self) -> [String; 4] {
        [
            VERB.to_owned(),
            self.task.to_string(),
            self.started.to_string(),
            self.value.to_string(),
        ]
    }

    pub fn parse(arguments: &[String]) -> Result<Self, Refusal> {
        let [verb, task, started, value] = arguments else {
            return Err(Refusal::Usage);
        };
        if verb != VERB {
            return Err(Refusal::Usage);
        }
        Ok(Self {
            task: number(task)?,
            started: number(started)?,
            value: number(value)?,
        })
    }
}

fn number<T: std::str::FromStr>(text: &str) -> Result<T, Refusal> {
    text.parse().map_err(|_| Refusal::Usage)
}
