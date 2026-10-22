use std::cell::Cell;
use std::rc::Rc;

use glib::Object;
use glib::prelude::ObjectExt;
use tokio::sync::mpsc::UnboundedSender;

use crate::domain::Event;

pub fn connect(session: &Object, output: UnboundedSender<Event>, done: Rc<Cell<bool>>) {
    let requests = output.clone();
    session.connect_local("request", false, move |values| {
        let text = values
            .get(1)
            .and_then(|value| value.get::<String>().ok())
            .unwrap_or_default();
        let echo = values
            .get(2)
            .and_then(|value| value.get::<bool>().ok())
            .unwrap_or(false);
        let _ = requests.send(Event::Prompt { text, echo });
        None
    });
    for (signal, error) in [("show-info", false), ("show-error", true)] {
        let notices = output.clone();
        session.connect_local(signal, false, move |values| {
            let text = values
                .get(1)
                .and_then(|value| value.get::<String>().ok())
                .unwrap_or_default();
            let event = match error {
                true => Event::Problem(text),
                false => Event::Info(text),
            };
            let _ = notices.send(event);
            None
        });
    }
    session.connect_local("completed", false, move |values| {
        done.set(true);
        let gained = values
            .get(1)
            .and_then(|value| value.get::<bool>().ok())
            .unwrap_or(false);
        let _ = output.send(Event::Completed(gained));
        None
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Secret;

    #[test]
    fn sinais_nativos_entregam_prompt_mensagens_e_conclusao_sem_iniciar_pam() {
        let api = super::super::api::api().unwrap();
        let session = api
            .session(0, &Secret::new("fixture-sem-pam".to_owned()))
            .unwrap();
        let (sender, mut events) = tokio::sync::mpsc::unbounded_channel();
        let done = Rc::new(Cell::new(false));
        connect(&session, sender, done.clone());
        session.emit_by_name::<()>("request", &[&"Senha:", &false]);
        assert!(
            matches!(events.try_recv(), Ok(Event::Prompt { text, echo: false }) if text == "Senha:")
        );
        session.emit_by_name::<()>("show-info", &[&"Informação"]);
        assert!(matches!(events.try_recv(), Ok(Event::Info(text)) if text == "Informação"));
        session.emit_by_name::<()>("show-error", &[&"Recusada"]);
        assert!(matches!(events.try_recv(), Ok(Event::Problem(text)) if text == "Recusada"));
        session.emit_by_name::<()>("completed", &[&false]);
        assert!(matches!(events.try_recv(), Ok(Event::Completed(false))));
        assert!(done.get());
    }
}
