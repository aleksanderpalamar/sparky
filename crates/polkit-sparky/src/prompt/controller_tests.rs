use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicUsize, Ordering},
};

use super::{Message, Window, controller, result};
use crate::domain::{Event, Form, Identity, Outcome, Request, Secret};
use crate::ports::{Authenticator, Conversation, Responses};

#[derive(Default)]
struct Backend {
    dropped: Arc<AtomicUsize>,
}

struct Control(Arc<AtomicUsize>);

impl Responses for Control {
    fn respond(&self, _: Secret) -> Result<(), String> {
        Ok(())
    }
}

impl Drop for Control {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

impl Authenticator for Backend {
    fn start(&self, _: u32, _: &Secret) -> Result<Conversation, String> {
        let (_, events) = tokio::sync::mpsc::unbounded_channel();
        Ok(Conversation {
            responses: Box::new(Control(self.dropped.clone())),
            events,
        })
    }

    fn shutdown(&self) {}
}

fn window() -> (Window, Arc<Backend>) {
    let request = Request {
        action_id: "org.fixture".to_owned(),
        message: "Autentique".to_owned(),
        cookie: Secret::new("fixture".to_owned()),
        identities: vec![Identity {
            uid: 0,
            name: "root".to_owned(),
        }],
    };
    let backend = Arc::new(Backend::default());
    let mut window = Window {
        form: Form::new(request, 0).unwrap(),
        skin: Default::default(),
        backend: backend.clone(),
        responses: None,
        generation: 0,
        outcome: Arc::new(AtomicU8::new(Outcome::Cancelled.code())),
    };
    let _ = controller::start(&mut window);
    (window, backend)
}

#[test]
fn conclusao_antiga_nao_autoriza_e_fechar_cancela_a_conversa() {
    let (mut window, backend) = window();
    let _ = controller::update(&mut window, Message::Backend(0, Event::Completed(true)));
    assert_eq!(result(&window.outcome), Outcome::Cancelled);
    let _ = controller::update(&mut window, Message::Close);
    assert_eq!(backend.dropped.load(Ordering::SeqCst), 1);
    assert_eq!(result(&window.outcome), Outcome::Cancelled);
}

#[test]
fn trocar_tema_preserva_a_resposta_e_o_evento_atual_encerra_a_conversa() {
    let (mut window, backend) = window();
    let generation = window.generation;
    let _ = controller::update(
        &mut window,
        Message::Backend(
            generation,
            Event::Prompt {
                text: "Senha".to_owned(),
                echo: false,
            },
        ),
    );
    let _ = controller::update(
        &mut window,
        Message::Typed(Secret::new("privada".to_owned())),
    );
    let palette = sparky_theme::Palette::from_colors("background = \"#ffffff\"");
    let _ = controller::update(&mut window, Message::Palette(palette));
    assert_eq!(window.form.response(), "privada");
    assert_eq!(window.skin.background, iced::Color::WHITE);
    let _ = controller::update(
        &mut window,
        Message::Backend(generation, Event::Completed(true)),
    );
    assert!(window.form.response().is_empty());
    assert_eq!(result(&window.outcome), Outcome::Authorized);
    assert_eq!(backend.dropped.load(Ordering::SeqCst), 1);
}

#[test]
fn enviar_limpa_a_senha_e_reiniciar_descarta_o_controle_anterior() {
    let (mut window, backend) = window();
    window.form.receive(Event::Prompt {
        text: "Senha".to_owned(),
        echo: false,
    });
    let _ = controller::update(
        &mut window,
        Message::Typed(Secret::new("privada".to_owned())),
    );
    let _ = controller::update(&mut window, Message::Submit);
    assert!(window.form.response().is_empty());
    let previous = window.generation;
    let _ = controller::start(&mut window);
    assert!(window.generation > previous);
    assert_eq!(backend.dropped.load(Ordering::SeqCst), 1);
}
