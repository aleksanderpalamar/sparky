use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

use super::wire::{decode_outcome, decode_request, encode_outcome, encode_request};
use super::{Filter, Outcome, Purpose, Request, Rule};

fn pedido(purpose: Purpose) -> Request {
    Request {
        purpose,
        title: "File Upload - YouTube Studio".to_owned(),
        accept: Some("Enviar".to_owned()),
        folder: Some(PathBuf::from("/home/ana/Vídeos")),
        filters: vec![Filter {
            name: "Vídeos".to_owned(),
            rules: vec![
                Rule::Glob("*.mp4".to_owned()),
                Rule::Mime("video/*".to_owned()),
            ],
        }],
        filter: Some(0),
    }
}

#[test]
fn o_pedido_vai_e_volta_igual_para_os_tres_propositos() {
    for purpose in [
        Purpose::Open {
            multiple: true,
            directory: false,
        },
        Purpose::Save {
            name: "video.mp4".to_owned(),
        },
        Purpose::SaveFiles {
            names: vec!["a.png".to_owned(), "b.png".to_owned()],
        },
    ] {
        let original = pedido(purpose);
        assert_eq!(decode_request(&encode_request(&original)), Some(original));
    }
}

#[test]
fn caminho_que_nao_e_utf8_chega_intacto() {
    let estranho = PathBuf::from(OsString::from_vec(vec![
        b'/', b't', 0xff, b'.', b'm', b'p', b'4',
    ]));
    let resultado = Outcome {
        paths: vec![estranho],
        filter: None,
    };
    assert_eq!(decode_outcome(&encode_outcome(&resultado)), Some(resultado));
}

#[test]
fn texto_invalido_ou_filtro_fora_da_lista_nao_viram_pedido_aceito() {
    assert_eq!(decode_request("{}"), None);
    assert_eq!(decode_request("[general"), None);
    assert_eq!(decode_outcome(""), None);
    let mut fora = pedido(Purpose::Open {
        multiple: false,
        directory: false,
    });
    fora.filter = Some(7);
    assert_eq!(
        decode_request(&encode_request(&fora)).and_then(|pedido| pedido.filter),
        None
    );
}
