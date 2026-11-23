use super::message::Message;
use super::test_support::{Memory, Started, dock};
use super::update::update;
use crate::domain::pinned::NATIVE;
use crate::domain::support::{app, ids};

fn catalog() -> Vec<crate::domain::app::App> {
    vec![
        app("sparky-files", "Arquivos"),
        app("firefox", "Firefox"),
        app("steam", "Steam"),
    ]
}

#[test]
fn primeira_execucao_usa_os_nativos_sem_gravar_nada() {
    let memory = Memory::default();
    let dock = dock(&memory, &Started::default(), catalog());
    assert_eq!(dock.pinned.ids(), ids(&NATIVE));
    assert_eq!(*memory.saves.borrow(), 0);
}

#[test]
fn restaura_a_configuracao_salva_e_ignora_lixo() {
    let memory = Memory::default();
    *memory.stored.borrow_mut() = Some(ids(&["steam", "", "steam", "firefox"]));
    let dock = dock(&memory, &Started::default(), catalog());
    assert_eq!(dock.pinned.ids(), ids(&["steam", "firefox"]));
}

#[test]
fn configuracao_ilegivel_ou_so_com_lixo_volta_ao_padrao() {
    let broken = Memory {
        broken: true,
        ..Memory::default()
    };
    assert_eq!(
        dock(&broken, &Started::default(), Vec::new()).pinned.ids(),
        ids(&NATIVE)
    );
    let lixo = Memory::default();
    *lixo.stored.borrow_mut() = Some(ids(&["", "a/b"]));
    assert_eq!(
        dock(&lixo, &Started::default(), Vec::new()).pinned.ids(),
        ids(&NATIVE)
    );
}

#[test]
fn adicionar_selecionados_fixa_na_ordem_e_salva_uma_vez() {
    let memory = Memory::default();
    let mut dock = dock(&memory, &Started::default(), catalog());
    let _ = update(&mut dock, Message::OpenPicker);
    let _ = update(&mut dock, Message::Toggle("steam".to_owned()));
    let _ = update(&mut dock, Message::Toggle("firefox".to_owned()));
    let _ = update(&mut dock, Message::Confirm);
    assert!(dock.picker.is_none());
    assert_eq!(dock.pinned.ids()[4..], ids(&["steam", "firefox"]));
    assert_eq!(*memory.saves.borrow(), 1);
    assert_eq!(
        memory.stored.borrow().as_deref().map(<[String]>::len),
        Some(6)
    );
}

#[test]
fn cancelar_descarta_a_selecao_sem_gravar() {
    let memory = Memory::default();
    let mut dock = dock(&memory, &Started::default(), catalog());
    let _ = update(&mut dock, Message::OpenPicker);
    let _ = update(&mut dock, Message::Toggle("steam".to_owned()));
    let _ = update(&mut dock, Message::Cancel);
    assert!(dock.picker.is_none());
    assert!(!dock.pinned.contains("steam"));
    let _ = update(&mut dock, Message::OpenPicker);
    assert!(
        dock.picker
            .as_ref()
            .is_some_and(|open| open.picker.chosen().is_empty())
    );
    assert_eq!(*memory.saves.borrow(), 0);
}

#[test]
fn confirmar_sem_novidade_nao_regrava_a_configuracao() {
    let memory = Memory::default();
    let mut dock = dock(&memory, &Started::default(), catalog());
    let _ = update(&mut dock, Message::OpenPicker);
    let _ = update(&mut dock, Message::Toggle("sparky-files".to_owned()));
    let _ = update(&mut dock, Message::Confirm);
    assert_eq!(*memory.saves.borrow(), 0);
}

#[test]
fn clicar_abre_o_aplicativo_instalado_e_ignora_o_ausente() {
    let started = Started::default();
    let mut dock = dock(&Memory::default(), &started, catalog());
    let _ = update(&mut dock, Message::Start("firefox".to_owned()));
    let _ = update(&mut dock, Message::Start("sparky-store".to_owned()));
    let abertos: Vec<Vec<String>> = started
        .0
        .borrow()
        .iter()
        .map(|launch| launch.arguments.clone())
        .collect();
    assert_eq!(abertos, [ids(&["firefox"])]);
}

#[test]
fn o_dock_mostra_so_os_fixados_instalados_com_o_estado_das_janelas() {
    let mut dock = dock(&Memory::default(), &Started::default(), catalog());
    let seen = crate::domain::presence::Seen {
        app_id: "sparky-files".to_owned(),
        focused: true,
        visible: true,
    };
    let _ = update(&mut dock, Message::Windows(vec![seen]));
    let shown: Vec<(&str, crate::domain::presence::Presence)> = dock
        .entries()
        .into_iter()
        .map(|(app, presence)| (app.id.as_str(), presence))
        .collect();
    assert_eq!(
        shown,
        [("sparky-files", crate::domain::presence::Presence::Focused)]
    );
}
