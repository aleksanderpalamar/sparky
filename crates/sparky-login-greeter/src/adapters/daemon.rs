use std::io::BufReader;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::Mutex;

use sparky_login_core::codec::{receive, send};
use sparky_login_core::protocol::{Power, Refusal, Reply, Request};

use crate::domain::form::Credentials;
use crate::ports::backend::Backend;

const OUT_OF_TURN: &str = "o serviço de login respondeu fora de hora";
const SILENT: &str = "o serviço de login não respondeu";

pub struct Daemon {
    link: Mutex<Link>,
}

struct Link {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
}

pub type Welcome = (Option<String>, Option<String>);

impl Daemon {
    pub fn connect(path: &Path) -> Result<Self, String> {
        let stream =
            UnixStream::connect(path).map_err(|error| format!("{}: {error}", path.display()))?;
        Self::from_stream(stream)
    }

    pub fn from_stream(stream: UnixStream) -> Result<Self, String> {
        let writer = stream.try_clone().map_err(|error| error.to_string())?;
        Ok(Self {
            link: Mutex::new(Link {
                reader: BufReader::new(stream),
                writer,
            }),
        })
    }

    pub fn hello(&self) -> Result<Welcome, String> {
        match self.ask(&Request::Hello)? {
            Reply::Welcome {
                last_user,
                last_session,
            } => Ok((last_user, last_session)),
            _ => Err(OUT_OF_TURN.to_owned()),
        }
    }

    fn ask(&self, request: &Request) -> Result<Reply, String> {
        let mut link = self.link.lock().map_err(|_| SILENT.to_owned())?;
        send(&mut link.writer, request).map_err(|error| error.to_string())?;
        receive(&mut link.reader)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| SILENT.to_owned())
    }
}

impl Backend for Daemon {
    fn login(&self, credentials: Credentials, session: &str) -> Result<(), Refusal> {
        let request = Request::Login {
            username: credentials.username,
            password: credentials.password,
            session: session.to_owned(),
        };
        match self.ask(&request) {
            Ok(Reply::LoggedIn) => Ok(()),
            Ok(Reply::Refused { refusal }) => Err(refusal),
            Ok(Reply::Failed { message }) => Err(Refusal::Unavailable(message)),
            Ok(_) => Err(Refusal::Unavailable(OUT_OF_TURN.to_owned())),
            Err(error) => Err(Refusal::Unavailable(error)),
        }
    }

    fn power(&self, action: Power) -> Result<(), String> {
        match self.ask(&Request::Power { action })? {
            Reply::Done => Ok(()),
            Reply::Failed { message } => Err(message),
            _ => Err(OUT_OF_TURN.to_owned()),
        }
    }
}
