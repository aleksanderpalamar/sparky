use std::sync::{Arc, Mutex};

use crate::adapters::file_manager1::FileManager1;

type Abertas = Arc<Mutex<Vec<(Vec<String>, Option<String>)>>>;

fn servico() -> (FileManager1, Abertas) {
    let abertas: Abertas = Arc::default();
    let registro = abertas.clone();
    let servico = FileManager1::new(move |args: Vec<String>, token: Option<String>| {
        if let Ok(mut lista) = registro.lock() {
            lista.push((args, token));
        }
    });
    (servico, abertas)
}

fn vistas(abertas: &Abertas) -> Vec<(Vec<String>, Option<String>)> {
    abertas
        .lock()
        .map(|lista| lista.clone())
        .unwrap_or_default()
}

#[test]
fn mostrar_itens_abre_a_pasta_com_selecao_e_repassa_o_token() {
    let (servico, abertas) = servico();
    servico.show_items(vec!["file:///tmp/a.txt".to_owned()], "token-1".to_owned());
    assert_eq!(
        vistas(&abertas),
        [(
            vec!["--select".to_owned(), "/tmp/a.txt".to_owned()],
            Some("token-1".to_owned())
        )]
    );
}

#[test]
fn pastas_e_propriedades_sem_token_abrem_janelas_proprias() {
    let (servico, abertas) = servico();
    servico.show_folders(vec!["file:///srv".to_owned()], String::new());
    servico.show_item_properties(vec!["file:///srv/x".to_owned()], String::new());
    assert_eq!(
        vistas(&abertas),
        [
            (vec!["/srv".to_owned()], None),
            (vec!["--properties".to_owned(), "/srv/x".to_owned()], None),
        ]
    );
}
