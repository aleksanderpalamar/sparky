use crate::domain::item::{Attributes, Edit, NewItem};
use crate::domain::secret::Secret;
use crate::domain::vault::{Existing, Stored, Vault};

fn atributos(pares: &[(&str, &str)]) -> Attributes {
    pares.iter().map(|(nome, valor)| (nome.to_string(), valor.to_string())).collect()
}

fn novo(rotulo: &str, pares: &[(&str, &str)], segredo: &str) -> NewItem {
    NewItem { label: rotulo.to_owned(), attributes: atributos(pares), secret: Secret::new(segredo.as_bytes().to_vec(), "text/plain") }
}

#[test]
fn guardar_cria_itens_com_numeros_novos() {
    let mut cofre = Vault::new(10);
    assert_eq!(cofre.store(novo("git", &[("service", "git")], "a"), Existing::Keep, 20), Stored::Created(1));
    assert_eq!(cofre.store(novo("git", &[("service", "git")], "b"), Existing::Keep, 30), Stored::Created(2));
    assert_eq!(cofre.items().len(), 2);
    assert_eq!(cofre.modified, 30);
    assert_eq!(cofre.item(1).map(|item| item.created), Some(20));
}

#[test]
fn substituir_troca_o_item_com_os_mesmos_atributos() {
    let mut cofre = Vault::new(0);
    cofre.store(novo("antigo", &[("service", "git"), ("user", "ana")], "velha"), Existing::Keep, 1);
    let guardado = cofre.store(novo("novo", &[("user", "ana"), ("service", "git")], "nova"), Existing::Replace, 5);
    assert_eq!(guardado, Stored::Replaced(1));
    let item = cofre.item(1);
    assert_eq!(item.map(|item| item.label.as_str()), Some("novo"));
    assert_eq!(item.map(|item| item.secret.value.as_slice()), Some(b"nova".as_slice()));
    assert_eq!(item.map(|item| (item.created, item.modified)), Some((1, 5)));
}

#[test]
fn substituir_sem_item_igual_cria_um_novo() {
    let mut cofre = Vault::new(0);
    cofre.store(novo("a", &[("service", "git"), ("user", "ana")], "x"), Existing::Keep, 1);
    assert_eq!(cofre.store(novo("b", &[("service", "git")], "y"), Existing::Replace, 2), Stored::Created(2));
}

#[test]
fn a_busca_exige_todos_os_atributos_pedidos() {
    let mut cofre = Vault::new(0);
    cofre.store(novo("a", &[("service", "git"), ("user", "ana")], "x"), Existing::Keep, 1);
    cofre.store(novo("b", &[("service", "git"), ("user", "bia")], "y"), Existing::Keep, 1);
    assert_eq!(cofre.search(&atributos(&[("service", "git")])), vec![1, 2]);
    assert_eq!(cofre.search(&atributos(&[("service", "git"), ("user", "bia")])), vec![2]);
    assert!(cofre.search(&atributos(&[("service", "ssh")])).is_empty());
    assert_eq!(cofre.search(&Attributes::new()), vec![1, 2]);
}

#[test]
fn apagar_e_editar_so_valem_para_itens_que_existem() {
    let mut cofre = Vault::new(0);
    cofre.store(novo("a", &[("service", "git")], "x"), Existing::Keep, 1);
    assert!(cofre.edit(1, Edit::Label("rótulo".to_owned()), 7));
    assert_eq!(cofre.item(1).map(|item| (item.label.as_str(), item.modified)), Some(("rótulo", 7)));
    assert!(!cofre.edit(9, Edit::Label("x".to_owned()), 7));
    assert!(cofre.delete(1, 8));
    assert!(!cofre.delete(1, 9));
    assert!(cofre.items().is_empty());
}

#[test]
fn os_numeros_continuam_depois_do_maior_ao_restaurar() {
    let mut cofre = Vault::new(0);
    cofre.store(novo("a", &[], "x"), Existing::Keep, 1);
    cofre.store(novo("b", &[], "y"), Existing::Keep, 1);
    cofre.delete(1, 2);
    let mut restaurado = Vault::restore(0, 2, cofre.items().to_vec());
    assert_eq!(restaurado.store(novo("c", &[], "z"), Existing::Keep, 3), Stored::Created(3));
}
