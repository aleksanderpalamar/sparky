use crate::domain::entry::{Kind, entry};
use crate::domain::listing::{Hidden, Listing};

fn pasta() -> Listing {
    Listing::new(
        vec![
            entry("notas.txt", Kind::File),
            entry(".config", Kind::Directory),
            entry("Fotos", Kind::Directory),
            entry("arquivo.pdf", Kind::File),
        ],
        Hidden::Concealed,
    )
}

fn nomes(listing: &Listing) -> Vec<&str> {
    listing.visible().map(|entry| entry.name.as_str()).collect()
}

fn selecionado(listing: &Listing) -> Option<&str> {
    listing.selected().map(|entry| entry.name.as_str())
}

#[test]
fn esconde_os_ocultos_e_mostra_quando_pedido_mantendo_a_selecao() {
    let mut listing = pasta();
    assert_eq!(nomes(&listing), ["Fotos", "arquivo.pdf", "notas.txt"]);
    listing.move_by(1);
    listing.toggle_hidden();
    assert_eq!(nomes(&listing), [".config", "Fotos", "arquivo.pdf", "notas.txt"]);
    assert_eq!(selecionado(&listing), Some("arquivo.pdf"));
}

#[test]
fn o_cursor_para_nas_pontas() {
    let mut listing = pasta();
    listing.move_by(-5);
    assert_eq!(listing.cursor(), 0);
    listing.move_by(50);
    assert_eq!(selecionado(&listing), Some("notas.txt"));
    listing.first();
    assert_eq!(selecionado(&listing), Some("Fotos"));
    listing.last();
    assert_eq!(selecionado(&listing), Some("notas.txt"));
}

#[test]
fn o_filtro_ignora_caixa_e_volta_o_cursor_ao_comeco() {
    let mut listing = pasta();
    listing.last();
    listing.set_filter("ARQ");
    assert_eq!(nomes(&listing), ["arquivo.pdf"]);
    assert_eq!(listing.cursor(), 0);
    listing.set_filter("");
    assert_eq!(listing.len(), 3);
}

#[test]
fn pasta_vazia_nao_tem_selecao() {
    let mut listing = Listing::new(Vec::new(), Hidden::Concealed);
    listing.move_by(1);
    listing.last();
    assert_eq!(listing.selected(), None);
    assert!(listing.chosen().is_empty());
}

#[test]
fn sem_marcas_a_escolha_e_o_selecionado() {
    let mut listing = pasta();
    listing.move_by(2);
    assert_eq!(listing.chosen(), ["notas.txt"]);
}

#[test]
fn marcar_de_novo_desmarca_e_as_marcas_viram_a_escolha() {
    let mut listing = pasta();
    listing.toggle_mark();
    listing.move_by(2);
    listing.toggle_mark();
    assert_eq!(listing.chosen(), ["Fotos", "notas.txt"]);
    listing.toggle_mark();
    assert_eq!(listing.chosen(), ["Fotos"]);
    listing.clear_marks();
    assert_eq!(listing.marked_count(), 0);
}

#[test]
fn recarregar_mantem_a_selecao_e_descarta_marcas_que_sumiram() {
    let mut listing = pasta();
    listing.toggle_mark();
    listing.move_by(1);
    listing.replace(vec![entry("arquivo.pdf", Kind::File), entry("novo.txt", Kind::File)]);
    assert_eq!(selecionado(&listing), Some("arquivo.pdf"));
    assert_eq!(listing.marked_count(), 0);
}

#[test]
fn selecionar_pelo_nome_e_pelo_indice() {
    let mut listing = pasta();
    listing.select_name("notas.txt");
    assert_eq!(listing.cursor(), 2);
    listing.select(9);
    assert_eq!(listing.cursor(), 2);
    listing.select(1);
    assert_eq!(selecionado(&listing), Some("arquivo.pdf"));
}
