use std::sync::Arc;

use iced::{Point, Settings, Size};
use iced_test::selector::Target;
use iced_test::{Simulator, selector};
use sparky_login_core::protocol::Refusal;
use sparky_login_look::layout::{Layout, Part, bands};
use sparky_login_look::style::REGULAR;
use zeroize::Zeroizing;

use crate::app::update::{PASSWORD, USER};
use crate::app::view::screen;
use crate::app::{Locker, Message, Start, boot};
use crate::ports::authenticator::Authenticator;

const LARGURA: f32 = 1920.0;
const ALTURA: f32 = 1080.0;

struct Recusa;

impl Authenticator for Recusa {
    fn unlock(&self, _password: Zeroizing<String>) -> Result<(), Refusal> {
        Err(Refusal::Denied)
    }
}

fn bloqueio() -> Locker {
    let start = Start {
        user: "palamar".into(),
        wallpaper: None,
        palette: sparky_theme::Palette::TOKYO_NIGHT,
        authenticator: Arc::new(Recusa),
    };
    boot(start).0
}

fn simulador(locker: &Locker) -> Simulator<'_, Message> {
    let settings = Settings {
        default_font: REGULAR,
        ..Settings::default()
    };
    Simulator::with_size(
        settings,
        (LARGURA, ALTURA),
        screen(locker, Size::new(LARGURA, ALTURA)),
    )
}

fn mensagens_ao_clicar(locker: &Locker, alvo: &str) -> Vec<Message> {
    let mut ui = simulador(locker);
    let _ = ui.click(alvo);
    ui.into_messages().collect()
}

#[test]
fn o_campo_de_usuario_mostra_o_nome_de_quem_bloqueou() {
    let locker = bloqueio();
    let mut ui = simulador(&locker);
    let achado = ui.find(selector::id(USER));
    assert!(matches!(achado, Ok(Target::TextInput { content, .. }) if content == "palamar"));
}

#[test]
fn o_unlock_so_responde_com_senha() {
    let mut locker = bloqueio();
    assert!(
        !mensagens_ao_clicar(&locker, "Desbloquear")
            .iter()
            .any(|message| matches!(message, Message::Submit))
    );
    locker.unlock.type_password("segredo".into());
    assert!(
        mensagens_ao_clicar(&locker, "Desbloquear")
            .iter()
            .any(|message| matches!(message, Message::Submit))
    );
}

#[test]
fn digitar_na_senha_chega_ao_formulario() {
    let locker = bloqueio();
    let mut ui = simulador(&locker);
    let _ = ui.click(selector::id(PASSWORD));
    let _ = ui.typewrite("abc");
    let digitado = ui.into_messages().filter_map(|message| match message {
        Message::Typed(text) => Some(text),
        _ => None,
    });
    assert_eq!(digitado.last().as_deref(), Some("abc"));
}

#[test]
fn clicar_no_olho_mostra_a_senha() {
    let locker = bloqueio();
    let layout = Layout::new(LARGURA, ALTURA);
    let centro = bands(ALTURA)
        .iter()
        .find(|band| band.part == Part::Password)
        .map_or(0.0, |band| band.top + band.height / 2.0);
    let mut ui = simulador(&locker);
    ui.point_at(Point::new(
        (layout.form_width - layout.field_width) / 2.0 + layout.field_height / 2.0,
        centro,
    ));
    let _ = ui.simulate(iced_test::simulator::click());
    assert!(
        ui.into_messages()
            .any(|message| matches!(message, Message::Reveal))
    );
}
