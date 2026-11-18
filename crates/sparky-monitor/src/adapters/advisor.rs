use std::path::PathBuf;

use sparky_resources::protocol::{Reply, Request, ask, socket};

use crate::domain::intelligence::{Advice, Briefing};
use crate::ports::Advisor;

pub struct ResourcesAdvisor {
    path: Option<PathBuf>,
}

impl ResourcesAdvisor {
    pub fn from_env() -> Self {
        Self {
            path: socket::from_env(),
        }
    }
}

impl Advisor for ResourcesAdvisor {
    fn advise(&self) -> Advice {
        self.path
            .as_ref()
            .and_then(|path| briefing(path))
            .map_or(Advice::Unavailable, |briefing| {
                Advice::Ready(Box::new(briefing))
            })
    }
}

fn briefing(path: &std::path::Path) -> Option<Briefing> {
    let Ok(Reply::Status(status)) = ask(path, Request::Status) else {
        return None;
    };
    let metrics = match ask(path, Request::Insight) {
        Ok(Reply::Insight { metrics }) => metrics,
        _ => Vec::new(),
    };
    let decisions = match ask(path, Request::Decisions) {
        Ok(Reply::Decisions { entries }) => entries,
        _ => Vec::new(),
    };
    Some(Briefing {
        status,
        metrics,
        decisions,
    })
}
