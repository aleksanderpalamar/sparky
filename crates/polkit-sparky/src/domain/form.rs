use super::{Event, Identity, Outcome, Request, Secret};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Waiting,
    Input { echo: bool },
    Verifying,
    Retry,
    Failed,
}

#[derive(Clone)]
pub struct Form {
    pub request: Request,
    pub identity: Identity,
    pub phase: Phase,
    pub prompt: String,
    pub problem: Option<String>,
    pub info: Option<String>,
    response: Secret,
    attempts: u8,
}

impl Form {
    pub fn new(request: Request, preferred: u32) -> Result<Self, String> {
        request.validate()?;
        let identity = request
            .identities
            .iter()
            .find(|identity| identity.uid == preferred)
            .or_else(|| request.identities.first())
            .cloned()
            .ok_or("Nenhum usuário disponível")?;
        Ok(Self {
            request,
            identity,
            phase: Phase::Waiting,
            prompt: String::new(),
            problem: None,
            info: None,
            response: Secret::default(),
            attempts: 0,
        })
    }

    pub fn response(&self) -> &str {
        self.response.expose()
    }

    pub fn select(&mut self, uid: u32) -> bool {
        let Some(identity) = self
            .request
            .identities
            .iter()
            .find(|identity| identity.uid == uid)
        else {
            return false;
        };
        self.identity = identity.clone();
        self.attempts = 0;
        self.reset();
        true
    }

    pub fn type_in(&mut self, response: Secret) {
        if !matches!(self.phase, Phase::Input { .. }) {
            return;
        }
        if response.expose().len() > 4096 || response.expose().contains(['\0', '\n', '\r']) {
            self.response = Secret::default();
            self.problem = Some("Resposta inválida".to_owned());
            return;
        }
        self.response = response;
        self.problem = None;
    }

    pub fn submit(&mut self) -> Option<Secret> {
        if !matches!(self.phase, Phase::Input { .. }) {
            return None;
        }
        self.phase = Phase::Verifying;
        Some(std::mem::take(&mut self.response))
    }

    pub fn retry(&mut self) -> bool {
        if self.phase != Phase::Retry {
            return false;
        }
        self.reset();
        true
    }

    pub fn receive(&mut self, event: Event) -> Option<Outcome> {
        match event {
            Event::Prompt { text, echo } => {
                self.response = Secret::default();
                self.prompt = text;
                self.phase = Phase::Input { echo };
            }
            Event::Info(text) => self.info = Some(text),
            Event::Problem(text) => self.problem = Some(text),
            Event::Failed(text) => {
                self.response = Secret::default();
                self.problem = Some(text);
                self.phase = Phase::Failed;
            }
            Event::Completed(true) => {
                self.response = Secret::default();
                return Some(Outcome::Authorized);
            }
            Event::Completed(false) => {
                self.response = Secret::default();
                self.attempts += 1;
                if self.attempts >= 3 {
                    return Some(Outcome::Denied);
                }
                self.phase = Phase::Retry;
                self.problem = Some("Autenticação não concluída. Tente novamente.".to_owned());
            }
        }
        None
    }

    fn reset(&mut self) {
        self.response = Secret::default();
        self.problem = None;
        self.info = None;
        self.phase = Phase::Waiting;
    }
}
