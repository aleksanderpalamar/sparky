use super::cover::{Crop, cover, left};

#[test]
fn imagem_mais_larga_que_a_tela_perde_as_laterais() {
    assert_eq!(
        cover((4000, 2000), (1920.0, 1080.0)),
        Crop {
            x: 222,
            y: 0,
            width: 3556,
            height: 2000
        }
    );
}

#[test]
fn imagem_mais_alta_que_a_tela_perde_em_cima_e_embaixo() {
    assert_eq!(
        cover((1920, 1440), (1920.0, 1080.0)),
        Crop {
            x: 0,
            y: 180,
            width: 1920,
            height: 1080
        }
    );
}

#[test]
fn mesma_proporcao_usa_a_imagem_inteira() {
    assert_eq!(
        cover((3840, 2160), (1920.0, 1080.0)),
        Crop {
            x: 0,
            y: 0,
            width: 3840,
            height: 2160
        }
    );
}

#[test]
fn tela_ainda_sem_tamanho_usa_a_imagem_inteira() {
    assert_eq!(
        cover((800, 600), (0.0, 0.0)),
        Crop {
            x: 0,
            y: 0,
            width: 800,
            height: 600
        }
    );
}

#[test]
fn a_faixa_da_esquerda_e_a_parte_do_recorte_atras_do_formulario() {
    let crop = Crop {
        x: 222,
        y: 0,
        width: 3556,
        height: 2000,
    };
    assert_eq!(
        left(crop, 0.4),
        Crop {
            x: 222,
            y: 0,
            width: 1422,
            height: 2000
        }
    );
}

#[test]
fn a_faixa_acompanha_a_copia_reduzida() {
    let crop = Crop {
        x: 200,
        y: 100,
        width: 3600,
        height: 2000,
    };
    assert_eq!(
        crop.scaled(0.25),
        Crop {
            x: 50,
            y: 25,
            width: 900,
            height: 500
        }
    );
}
