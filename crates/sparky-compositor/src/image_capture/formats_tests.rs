use super::formats::Formats;
use smithay::backend::allocator::{Format, Fourcc, Modifier};

#[test]
fn anuncio_usa_apenas_formatos_renderizaveis_de_oito_bits() {
    let mut formats = Formats::default();
    formats.set(
        None,
        [
            Format {
                code: Fourcc::Argb8888,
                modifier: Modifier::Linear,
            },
            Format {
                code: Fourcc::Abgr2101010,
                modifier: Modifier::Linear,
            },
            Format {
                code: Fourcc::Nv12,
                modifier: Modifier::Linear,
            },
        ],
    );
    assert_eq!(formats.entries.len(), 1);
    assert_eq!(formats.legacy(), Some(Fourcc::Argb8888));
}

#[test]
fn anuncio_legado_nao_inventa_um_formato_indisponivel() {
    let mut formats = Formats::default();
    assert_eq!(formats.legacy(), None);
    formats.set(
        None,
        [Format {
            code: Fourcc::Abgr8888,
            modifier: Modifier::Linear,
        }],
    );
    assert_eq!(formats.legacy(), None);
    formats.set(
        None,
        [Format {
            code: Fourcc::Xrgb8888,
            modifier: Modifier::Linear,
        }],
    );
    assert_eq!(formats.legacy(), Some(Fourcc::Xrgb8888));
}
