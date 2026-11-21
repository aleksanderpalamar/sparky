use super::request::{Refusal, Renice};

const HIGHEST: i8 = 0;
const LOWEST: i8 = 19;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Owner {
    pub real: u32,
    pub effective: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Facts {
    pub caller: Option<u32>,
    pub owner: Option<Owner>,
    pub started: Option<u64>,
}

pub fn owner(status: &str) -> Option<Owner> {
    let line = status.lines().find_map(|line| line.strip_prefix("Uid:"))?;
    let mut ids = line.split_whitespace().map(str::parse::<u32>);
    let real = ids.next()?.ok()?;
    let effective = ids.next()?.ok()?;
    Some(Owner { real, effective })
}

pub fn admit(request: &Renice, facts: &Facts) -> Result<(), Refusal> {
    if !(HIGHEST..=LOWEST).contains(&request.value) {
        return Err(Refusal::OutOfRange);
    }
    if request.task <= 1 {
        return Err(Refusal::Protected);
    }
    let caller = facts
        .caller
        .filter(|caller| *caller != 0)
        .ok_or(Refusal::Protected)?;
    if facts.started != Some(request.started) {
        return Err(Refusal::Gone);
    }
    let owner = facts.owner.ok_or(Refusal::Gone)?;
    if owner.real != caller || owner.effective != caller {
        return Err(Refusal::NotOwner);
    }
    Ok(())
}
