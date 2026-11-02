use std::sync::Arc;

use iced::{Point, Settings};
use iced_test::{Simulator, selector};

use crate::adapters::preview::Preview;
use crate::app::update::PASSWORD;
use crate::app::{Greeter, Message, Start, boot, style, view};
use crate::domain::layout::{Layout, Part, bands};
use crate::domain::power::Power;
use crate::domain::sessions::{Session, Sessions};

const LARGURA: f32 = 1920.0;
const ALTURA: f32 = 1080.0;

fn tela(last_user: Option<&str>) -> Greeter {
    let sessions = Sessions::new(
        vec![Session {
            id: "sparky".into(),
            name: "Sparky".into(),
        }],
        None,
    );
    let start = Start {
        backend: Arc::new(Preview),
        sessions,
        last_user: last_user.map(str::to_owned),
        wallpaper: None,
        palette: sparky_theme::Palette::TOKYO_NIGHT,
    };
    boot(start).0
}

fn simulador(greeter: &Greeter) -> Simulator<'_, Message> {
    let settings = Settings {
        default_font: style::REGULAR,
        ..Settings::default()
    };
    Simulator::with_size(settings, (LARGURA, ALTURA), view::view(greeter))
}

fn centro(part: Part) -> f32 {
    bands(ALTURA)
        .iter()
        .find(|band| band.part == part)
        .map_or(0.0, |band| band.top + band.height / 2.0)
}

fn clicar(greeter: &Greeter, ponto: Point) -> Vec<Message> {
    let mut ui = simulador(greeter);
    ui.point_at(ponto);
    let _ = ui.simulate(iced_test::simulator::click());
    ui.into_messages().collect()
}

#[test]
fn o_login_so_responde_com_usuario_e_senha() {
    let mut greeter = tela(Some("palamar"));
    let mut ui = simulador(&greeter);
    let _ = ui.click("Login");
    assert!(
        !ui.into_messages()
            .any(|message| matches!(message, Message::Login))
    );
    greeter.form.type_password("segredo".to_owned());
    let mut ui = simulador(&greeter);
    let _ = ui.click("Login");
    assert!(
        ui.into_messages()
            .any(|message| matches!(message, Message::Login))
    );
}

#[test]
fn digitar_na_senha_chega_ao_formulario() {
    let greeter = tela(Some("palamar"));
    let mut ui = simulador(&greeter);
    let _ = ui.click(selector::id(PASSWORD));
    let _ = ui.typewrite("abc");
    let ultima = ui.into_messages().filter_map(|message| match message {
        Message::TypedPassword(text) => Some(text),
        _ => None,
    });
    assert_eq!(ultima.last().as_deref(), Some("abc"));
}

#[test]
fn clicar_no_olho_mostra_a_senha() {
    let greeter = tela(Some("palamar"));
    let layout = Layout::new(LARGURA, ALTURA);
    let olho = Point::new(
        (layout.form_width - layout.field_width) / 2.0 + layout.field_height / 2.0,
        centro(Part::Password),
    );
    assert!(
        clicar(&greeter, olho)
            .iter()
            .any(|message| matches!(message, Message::Reveal))
    );
}

#[test]
fn os_icones_de_energia_pedem_a_acao_certa() {
    let greeter = tela(Some("palamar"));
    let layout = Layout::new(LARGURA, ALTURA);
    let passo = layout.power_width + layout.power_spacing;
    let meio = layout.form_width / 2.0;
    for (indice, action) in Power::ALL.into_iter().enumerate() {
        let ponto = Point::new(
            meio + (indice as f32 - 1.0) * passo,
            centro(Part::PowerIcons),
        );
        let mensagens = clicar(&greeter, ponto);
        assert!(
            mensagens
                .iter()
                .any(|message| matches!(message, Message::Power(found) if *found == action)),
            "{action:?}"
        );
    }
}

#[test]
fn clicar_na_sessao_troca_a_sessao() {
    let greeter = tela(Some("palamar"));
    let layout = Layout::new(LARGURA, ALTURA);
    let mensagens = clicar(
        &greeter,
        Point::new(layout.form_width / 2.0, centro(Part::Session)),
    );
    assert!(
        mensagens
            .iter()
            .any(|message| matches!(message, Message::NextSession))
    );
}
