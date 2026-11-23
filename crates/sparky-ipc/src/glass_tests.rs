use crate::codec::{decode, encode};
use crate::glass::{GlassArea, GlassRect};
use crate::message::Request;

#[test]
fn o_pedido_de_vidro_viaja_com_o_namespace_e_a_area() {
    let pedidos = [
        Request::Glass {
            namespace: "sparky-launcher".to_owned(),
            area: GlassArea::Centered {
                width: 760,
                height: 520,
            },
        },
        Request::Glass {
            namespace: "sparky-dock".to_owned(),
            area: GlassArea::Rects {
                rects: vec![GlassRect {
                    x: 10,
                    y: 20,
                    width: 408,
                    height: 100,
                }],
                radius: 12,
            },
        },
    ];
    for pedido in pedidos {
        let Ok(linha) = encode(&pedido) else {
            panic!("deveria serializar");
        };
        assert_eq!(decode::<Request>(&linha).ok(), Some(pedido));
    }
    assert_eq!(GlassArea::default(), GlassArea::Whole);
}

#[test]
fn pedido_sem_raio_vale_cantos_retos() {
    let linha =
        r#"{"request":"glass","namespace":"sparky-dock","area":{"kind":"rects","rects":[]}}"#;
    assert_eq!(
        decode::<Request>(linha).ok(),
        Some(Request::Glass {
            namespace: "sparky-dock".to_owned(),
            area: GlassArea::Rects {
                rects: Vec::new(),
                radius: 0
            },
        })
    );
}
