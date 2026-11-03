use sparky_login_core::protocol::{Power, Refusal, Reply, Request};
use sparky_login_core::sessions::Session;
use zeroize::Zeroizing;

use crate::domain::memory::Memory;

const BUSY: &str = "já há um login em andamento";
const STOPPED: &str = "o login parou no meio";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Login {
    pub username: String,
    pub password: Zeroizing<String>,
    pub session: Session,
}

#[derive(Debug)]
pub enum Event {
    Started,
    Request(Request),
    Authenticated,
    Refused(Refusal),
    Powered(Result<(), String>),
    GreeterGone,
    SessionEnded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    StartGreeter,
    Reply(Reply),
    Authenticate(Login),
    Power(Power),
    Remember(Memory),
    StopGreeter,
    StartSession,
    Abandon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Phase {
    Greeting,
    Checking(Memory),
    HandingOver,
    Recovering,
    InSession,
}

pub struct Machine {
    sessions: Vec<Session>,
    memory: Memory,
    phase: Phase,
}

impl Machine {
    pub fn new(sessions: Vec<Session>, memory: Memory) -> Self {
        Self {
            sessions,
            memory,
            phase: Phase::Greeting,
        }
    }

    pub fn set_sessions(&mut self, sessions: Vec<Session>) {
        self.sessions = sessions;
    }

    pub fn on(&mut self, event: Event) -> Vec<Effect> {
        match (self.phase.clone(), event) {
            (_, Event::Started) => vec![Effect::StartGreeter],
            (Phase::Greeting, Event::Request(request)) => self.request(request),
            (Phase::Checking(_), Event::Request(_)) => reply(Reply::Failed {
                message: BUSY.to_owned(),
            }),
            (_, Event::Powered(result)) => reply(powered(result)),
            (Phase::Checking(_), Event::Refused(refusal)) => self.refuse(refusal),
            (Phase::Checking(memory), Event::Authenticated) => self.hand_over(memory),
            (Phase::Checking(_), Event::SessionEnded) => {
                self.refuse(Refusal::Unavailable(STOPPED.to_owned()))
            }
            (Phase::Checking(_), Event::GreeterGone) => {
                self.restart(vec![Effect::Abandon, Effect::StartGreeter])
            }
            (Phase::HandingOver, Event::GreeterGone) => {
                self.enter(Phase::InSession, vec![Effect::StartSession])
            }
            (Phase::HandingOver, Event::SessionEnded) => self.enter(Phase::Recovering, Vec::new()),
            (Phase::Greeting | Phase::Recovering, Event::GreeterGone) => {
                self.restart(vec![Effect::StartGreeter])
            }
            (Phase::InSession, Event::SessionEnded) => self.restart(vec![Effect::StartGreeter]),
            _ => Vec::new(),
        }
    }

    fn request(&mut self, request: Request) -> Vec<Effect> {
        match request {
            Request::Hello => reply(Reply::Welcome {
                last_user: self.memory.last_user.clone(),
                last_session: self.memory.last_session.clone(),
            }),
            Request::Power { action } => vec![Effect::Power(action)],
            Request::Login {
                username,
                password,
                session,
            } => self.check(username, password, &session),
        }
    }

    fn check(&mut self, username: String, password: Zeroizing<String>, id: &str) -> Vec<Effect> {
        let username = username.trim().to_owned();
        if username.is_empty() {
            return reply(Reply::Refused {
                refusal: Refusal::Denied,
            });
        }
        let Some(session) = self
            .sessions
            .iter()
            .find(|session| session.id == id)
            .cloned()
        else {
            let refusal = Refusal::Unavailable(format!("a sessão {id} não está instalada"));
            return reply(Reply::Refused { refusal });
        };
        let memory = Memory {
            last_user: Some(username.clone()),
            last_session: Some(session.id.clone()),
        };
        self.enter(
            Phase::Checking(memory),
            vec![Effect::Authenticate(Login {
                username,
                password,
                session,
            })],
        )
    }

    fn refuse(&mut self, refusal: Refusal) -> Vec<Effect> {
        self.enter(Phase::Greeting, reply(Reply::Refused { refusal }))
    }

    fn hand_over(&mut self, memory: Memory) -> Vec<Effect> {
        self.memory = memory.clone();
        let effects = vec![
            Effect::Reply(Reply::LoggedIn),
            Effect::Remember(memory),
            Effect::StopGreeter,
        ];
        self.enter(Phase::HandingOver, effects)
    }

    fn restart(&mut self, effects: Vec<Effect>) -> Vec<Effect> {
        self.enter(Phase::Greeting, effects)
    }

    fn enter(&mut self, phase: Phase, effects: Vec<Effect>) -> Vec<Effect> {
        self.phase = phase;
        effects
    }
}

fn reply(reply: Reply) -> Vec<Effect> {
    vec![Effect::Reply(reply)]
}

fn powered(result: Result<(), String>) -> Reply {
    match result {
        Ok(()) => Reply::Done,
        Err(message) => Reply::Failed { message },
    }
}
