use std::path::PathBuf;

use zbus::zvariant::{OwnedValue, Value};

use super::options::{Call, Options, request};
use super::results::results;
use crate::domain::choice::{Filter, Outcome, Purpose, Rule};

fn valor<'a>(value: impl Into<Value<'a>>) -> OwnedValue {
    match OwnedValue::try_from(value.into()) {
        Ok(value) => value,
        Err(error) => panic!("valor inválido: {error}"),
    }
}

fn videos() -> (String, Vec<(u32, String)>) {
    (
        "Vídeos".to_owned(),
        vec![(0, "*.mp4".to_owned()), (1, "video/*".to_owned())],
    )
}

fn opcoes(pares: Vec<(&str, OwnedValue)>) -> Options {
    pares
        .into_iter()
        .map(|(chave, valor)| (chave.to_owned(), valor))
        .collect()
}

#[test]
fn abrir_arquivo_le_rotulo_multiplos_pasta_e_filtro_escolhido() {
    let opcoes = opcoes(vec![
        ("accept_label", valor("_Enviar")),
        ("multiple", valor(true)),
        ("filters", valor(vec![videos()])),
        ("current_filter", valor(videos())),
        (
            "current_folder",
            valor(b"/home/ana/V\xc3\xaddeos\0".to_vec()),
        ),
    ]);
    let pedido = request(Call::OpenFile, "File Upload", &opcoes);
    assert_eq!(
        pedido.purpose,
        Purpose::Open {
            multiple: true,
            directory: false
        }
    );
    assert_eq!(pedido.title, "File Upload");
    assert_eq!(pedido.accept.as_deref(), Some("_Enviar"));
    assert_eq!(pedido.folder, Some(PathBuf::from("/home/ana/Vídeos")));
    assert_eq!(pedido.filter, Some(0));
    assert_eq!(
        pedido.filters,
        vec![Filter {
            name: "Vídeos".to_owned(),
            rules: vec![
                Rule::Glob("*.mp4".to_owned()),
                Rule::Mime("video/*".to_owned())
            ],
        }]
    );
}

#[test]
fn filtro_atual_sem_lista_vira_o_unico_filtro_e_opcoes_ausentes_tem_padrao() {
    let pedido = request(
        Call::OpenFile,
        "",
        &opcoes(vec![("current_filter", valor(videos()))]),
    );
    assert_eq!(pedido.filters.len(), 1);
    assert_eq!(pedido.filter, Some(0));
    let vazio = request(Call::OpenFile, "", &Options::new());
    assert_eq!(
        vazio.purpose,
        Purpose::Open {
            multiple: false,
            directory: false
        }
    );
    assert_eq!(
        (vazio.folder, vazio.accept, vazio.filter),
        (None, None, None)
    );
}

#[test]
fn salvar_usa_o_arquivo_atual_ou_o_nome_sugerido() {
    let com_arquivo = opcoes(vec![
        ("current_name", valor("ignorado.txt")),
        ("current_file", valor(b"/home/ana/notas.txt\0".to_vec())),
    ]);
    let pedido = request(Call::SaveFile, "Salvar", &com_arquivo);
    assert_eq!(
        pedido.purpose,
        Purpose::Save {
            name: "notas.txt".to_owned()
        }
    );
    assert_eq!(pedido.folder, Some(PathBuf::from("/home/ana")));
    let so_nome = request(
        Call::SaveFile,
        "Salvar",
        &opcoes(vec![("current_name", valor("foto.png"))]),
    );
    assert_eq!(
        so_nome.purpose,
        Purpose::Save {
            name: "foto.png".to_owned()
        }
    );
}

#[test]
fn salvar_varios_guarda_so_os_nomes() {
    let arquivos = vec![b"a.png\0".to_vec(), b"/tmp/b.png\0".to_vec()];
    let pedido = request(
        Call::SaveFiles,
        "",
        &opcoes(vec![("files", valor(arquivos))]),
    );
    assert_eq!(
        pedido.purpose,
        Purpose::SaveFiles {
            names: vec!["a.png".to_owned(), "b.png".to_owned()]
        }
    );
}

#[test]
fn o_resultado_traz_uris_e_o_filtro_usado() {
    let pedido = request(
        Call::OpenFile,
        "",
        &opcoes(vec![("filters", valor(vec![videos()]))]),
    );
    let saida = results(
        &pedido,
        &Outcome {
            paths: vec![PathBuf::from("/home/ana/Vídeos/demo sparky.mp4")],
            filter: Some(0),
        },
    );
    let Some(uris) = saida
        .get("uris")
        .and_then(|uris| Vec::<String>::try_from(uris.try_clone().ok()?).ok())
    else {
        panic!("sem uris");
    };
    assert_eq!(
        uris,
        vec!["file:///home/ana/V%C3%ADdeos/demo%20sparky.mp4".to_owned()]
    );
    assert!(saida.contains_key("current_filter"));
}
