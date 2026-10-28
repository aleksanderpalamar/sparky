use iced::keyboard::key::Named;
use sparky_theme::Skin;
use sparky_ui::notice::Notice;

use super::test_scene::Cena;
use crate::app::message::Message;
use crate::app::state::Mode;

#[test]
fn evento_de_paleta_preserva_uma_renomeacao_em_andamento() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.tecla("r");
    cena.mandar(Message::Typed("novo.txt".to_owned()));
    let mode = cena.files.mode.clone();
    let selected = cena.selecionado();
    let palette = sparky_theme::Palette::from_colors("accent = \"#010203\"");
    cena.mandar(Message::Palette(palette));
    assert_eq!(cena.files.skin, Skin::new(palette));
    assert_eq!(cena.files.mode, mode);
    assert_eq!(cena.files.input, "novo.txt");
    assert_eq!(cena.selecionado(), selected);
    assert!(cena.work().join("a.txt").exists());
}

#[test]
fn entrar_e_voltar_deixa_a_pasta_de_onde_veio_selecionada() {
    let mut cena = Cena::nova();
    cena.tecla("l");
    assert_eq!(cena.files.location, cena.work().join("Fotos"));
    cena.tecla("h");
    assert_eq!(cena.files.location, cena.work());
    assert_eq!(cena.selecionado().as_deref(), Some("Fotos"));
}

#[test]
fn criar_pasta_pelo_prompt_seleciona_a_nova() {
    let mut cena = Cena::nova();
    cena.tecla("a");
    assert_eq!(cena.files.mode, Mode::Create);
    cena.mandar(Message::Typed("Projetos/".to_owned()));
    cena.mandar(Message::Submit);
    assert!(cena.work().join("Projetos").is_dir());
    assert_eq!(cena.selecionado().as_deref(), Some("Projetos"));
    assert_eq!(cena.files.mode, Mode::Browse);
}

#[test]
fn renomear_comeca_com_o_nome_atual() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.tecla("r");
    assert_eq!(cena.files.input, "a.txt");
    cena.mandar(Message::Typed("b.txt".to_owned()));
    cena.mandar(Message::Submit);
    assert!(cena.work().join("b.txt").exists());
    assert_eq!(
        cena.files.notice,
        Some(Notice::Info("Renomeado para b.txt".to_owned()))
    );
}

#[test]
fn copiar_e_colar_na_mesma_pasta_duplica() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.tecla("y");
    cena.tecla("p");
    assert!(cena.work().join("a (cópia).txt").exists());
    assert_eq!(cena.selecionado().as_deref(), Some("a (cópia).txt"));
}

#[test]
fn a_lixeira_so_age_depois_da_confirmacao() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.tecla("d");
    assert_eq!(
        cena.files.mode,
        Mode::ConfirmTrash(vec!["a.txt".to_owned()])
    );
    cena.tecla("n");
    assert!(cena.work().join("a.txt").exists());
    cena.tecla("d");
    cena.nomeada(Named::Enter);
    assert!(!cena.work().join("a.txt").exists());
    assert!(cena.dir.path().join("Trash/files/a.txt").exists());
}

#[test]
fn o_filtro_reduz_a_lista_e_esc_limpa() {
    let mut cena = Cena::nova();
    cena.tecla("/");
    cena.mandar(Message::Typed("fo".to_owned()));
    assert_eq!(cena.files.listing.len(), 1);
    cena.mandar(Message::Escape);
    assert_eq!(cena.files.listing.len(), 2);
    assert_eq!(cena.files.mode, Mode::Browse);
}

#[test]
fn abrir_um_arquivo_chama_o_aplicativo_padrao() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.nomeada(Named::Enter);
    assert_eq!(cena.abertos(), vec![cena.work().join("a.txt")]);
}

#[test]
fn marcar_com_espaco_e_copiar_leva_todos_os_marcados() {
    let mut cena = Cena::nova();
    cena.nomeada(Named::Space);
    cena.nomeada(Named::Space);
    cena.tecla("y");
    assert_eq!(
        cena.files.clip.as_ref().map(|clip| clip.paths().len()),
        Some(2)
    );
    assert_eq!(cena.files.listing.marked_count(), 0);
}
