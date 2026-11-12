use iced::Point;

use super::message::Message;
use super::test_scene::Cena;
use crate::domain::thumbnail_key::file_uri;

fn indice(cena: &Cena, nome: &str) -> usize {
    cena.files
        .listing
        .visible()
        .position(|entry| entry.name == nome)
        .unwrap_or(usize::MAX)
}

fn pressionar(cena: &mut Cena, nome: &str) {
    cena.mandar(Message::Pointer(Point::new(10.0, 10.0)));
    let posicao = indice(cena, nome);
    cena.mandar(Message::Clicked(posicao));
}

fn mover(cena: &mut Cena, x: f32) {
    cena.concluir(Message::Pointer(Point::new(x, 10.0)));
}

fn uri(cena: &Cena, nome: &str) -> String {
    file_uri(&cena.work().join(nome))
}

#[test]
fn arrastar_dentro_da_janela_nao_chama_o_compositor() {
    let mut cena = Cena::nova();
    pressionar(&mut cena, "a.txt");
    mover(&mut cena, 60.0);
    assert!(cena.arrastos.pedidos().is_empty());
    assert!(cena.files.drag_status().is_some());
}

#[test]
fn sair_da_janela_arrastando_entrega_ao_compositor_uma_vez() {
    let mut cena = Cena::nova();
    pressionar(&mut cena, "a.txt");
    mover(&mut cena, 60.0);
    mover(&mut cena, -5.0);
    mover(&mut cena, -40.0);
    assert_eq!(cena.arrastos.pedidos(), [vec![uri(&cena, "a.txt")]]);
    assert!(cena.files.drag_status().is_none());
}

#[test]
fn sair_da_janela_sem_arrastar_nao_chama_o_compositor() {
    let mut cena = Cena::nova();
    pressionar(&mut cena, "a.txt");
    cena.mandar(Message::Released);
    mover(&mut cena, -40.0);
    assert!(cena.arrastos.pedidos().is_empty());
}

#[test]
fn sair_pela_direita_tambem_entrega_ao_compositor() {
    let mut cena = Cena::nova();
    pressionar(&mut cena, "a.txt");
    let largura = cena.files.size.width;
    mover(&mut cena, largura + 1.0);
    assert_eq!(cena.arrastos.pedidos(), [vec![uri(&cena, "a.txt")]]);
}

#[test]
fn arrastar_um_item_marcado_leva_todos_os_marcados_para_fora() {
    let mut cena = Cena::nova();
    cena.files.listing.select_all();
    pressionar(&mut cena, "Fotos");
    mover(&mut cena, 40.0);
    mover(&mut cena, -1.0);
    assert_eq!(
        cena.arrastos.pedidos(),
        [vec![uri(&cena, "Fotos"), uri(&cena, "a.txt")]]
    );
}

#[test]
fn recusa_do_compositor_aparece_na_barra_de_status() {
    let mut cena = Cena::nova();
    cena.arrastos.recusa = Some("fora da sessão".to_owned());
    cena.files.services.dragger = std::sync::Arc::new(cena.arrastos.clone());
    pressionar(&mut cena, "a.txt");
    mover(&mut cena, 40.0);
    mover(&mut cena, -1.0);
    let aviso = cena
        .files
        .notice
        .as_ref()
        .map(|notice| notice.text().to_owned());
    assert_eq!(
        aviso.as_deref(),
        Some("Não foi possível arrastar: fora da sessão")
    );
}
