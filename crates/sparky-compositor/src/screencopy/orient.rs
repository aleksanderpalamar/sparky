use smithay::utils::{Logical, Size, Transform};

const BYTES_PER_PIXEL: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixels {
    pub width: usize,
    pub rgba: Vec<u8>,
}

pub fn buffer_size(size: (i32, i32), transform: Transform) -> (i32, i32) {
    let turned = transform.transform_size(Size::<i32, Logical>::from(size));
    (turned.w, turned.h)
}

pub fn orient(rgba: &[u8], width: usize, transform: Transform) -> Option<Pixels> {
    let row = width * BYTES_PER_PIXEL;
    if row == 0 || !rgba.len().is_multiple_of(row) {
        return None;
    }
    let height = rgba.len() / row;
    let render = transform.invert();
    let turned = match render {
        Transform::_90 | Transform::_270 | Transform::Flipped90 | Transform::Flipped270 => height,
        _ => width,
    };
    let mut panel = vec![0; rgba.len()];
    for (index, pixel) in rgba.chunks_exact(BYTES_PER_PIXEL).enumerate() {
        let (x, y) = placed(render, index % width, index / width, width, height);
        let start = (y * turned + x) * BYTES_PER_PIXEL;
        panel[start..start + BYTES_PER_PIXEL].copy_from_slice(pixel);
    }
    Some(Pixels {
        width: turned,
        rgba: panel,
    })
}

fn placed(transform: Transform, x: usize, y: usize, width: usize, height: usize) -> (usize, usize) {
    let (right, bottom) = (width - 1 - x, height - 1 - y);
    match transform {
        Transform::Normal => (x, y),
        Transform::_90 => (bottom, x),
        Transform::_180 => (right, bottom),
        Transform::_270 => (y, right),
        Transform::Flipped => (right, y),
        Transform::Flipped90 => (y, x),
        Transform::Flipped180 => (x, bottom),
        Transform::Flipped270 => (bottom, right),
    }
}

#[cfg(test)]
mod tests {
    use super::{buffer_size, orient};
    use smithay::utils::{Logical, Point, Size, Transform};

    const IMAGEM: [u8; 16] = [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
    const GIROS: [Transform; 8] = [
        Transform::Normal,
        Transform::_90,
        Transform::_180,
        Transform::_270,
        Transform::Flipped,
        Transform::Flipped90,
        Transform::Flipped180,
        Transform::Flipped270,
    ];

    fn orientado(transform: Transform) -> Option<(usize, Vec<u8>)> {
        orient(&IMAGEM, 2, transform).map(|pixels| (pixels.width, pixels.rgba))
    }

    #[test]
    fn monitor_normal_recebe_a_imagem_como_esta() {
        assert_eq!(orientado(Transform::Normal), Some((2, IMAGEM.to_vec())));
    }

    #[test]
    fn monitor_espelhado_ou_girado_meia_volta_mantem_o_tamanho() {
        let linhas = vec![3, 3, 3, 3, 4, 4, 4, 4, 1, 1, 1, 1, 2, 2, 2, 2];
        assert_eq!(orientado(Transform::Flipped180), Some((2, linhas)));
        let colunas = vec![2, 2, 2, 2, 1, 1, 1, 1, 4, 4, 4, 4, 3, 3, 3, 3];
        assert_eq!(orientado(Transform::Flipped), Some((2, colunas)));
        let meia_volta = vec![4, 4, 4, 4, 3, 3, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1];
        assert_eq!(orientado(Transform::_180), Some((2, meia_volta)));
    }

    #[test]
    fn monitor_girado_de_lado_recebe_a_imagem_no_sentido_do_painel() {
        let largura = 4;
        let altura = 6;
        let mut logico = vec![0_u8; largura * altura * 4];
        logico[..4].copy_from_slice(&[9, 9, 9, 9]);
        logico[4..8].copy_from_slice(&[7, 7, 7, 7]);
        let Some(painel) = orient(&logico, largura, Transform::_90) else {
            panic!("giro de 90° suportado")
        };
        assert_eq!(painel.width, 6);
        let pixel = |x: usize, y: usize| painel.rgba[(y * 6 + x) * 4];
        assert_eq!(pixel(0, 3), 9);
        assert_eq!(pixel(0, 2), 7);
        let Some(outro_lado) = orient(&logico, largura, Transform::_270) else {
            panic!("giro de 270° suportado")
        };
        assert_eq!(outro_lado.width, 6);
        assert_eq!(outro_lado.rgba[5 * 4], 9);
        assert_eq!(outro_lado.rgba[(6 + 5) * 4], 7);
    }

    #[test]
    fn cada_pixel_vai_para_onde_o_smithay_desenha() {
        let (largura, altura) = (3_usize, 2_usize);
        let logico: Vec<u8> = (0..largura * altura)
            .flat_map(|indice| [indice as u8; 4])
            .collect();
        for giro in GIROS {
            let Some(painel) = orient(&logico, largura, giro) else {
                panic!("{giro:?} suportado")
            };
            let area = Size::<i32, Logical>::from((largura as i32 - 1, altura as i32 - 1));
            for indice in 0..largura * altura {
                let ponto = Point::<i32, Logical>::from((
                    (indice % largura) as i32,
                    (indice / largura) as i32,
                ));
                let destino = giro.invert().transform_point_in(ponto, &area);
                let posicao = destino.y as usize * painel.width + destino.x as usize;
                assert_eq!(painel.rgba[posicao * 4], indice as u8, "{giro:?}");
            }
        }
    }

    #[test]
    fn imagem_que_nao_fecha_linhas_e_recusada() {
        assert!(orient(&IMAGEM, 0, Transform::Normal).is_none());
        assert!(orient(&IMAGEM[..12], 2, Transform::_90).is_none());
    }

    #[test]
    fn o_buffer_anunciado_troca_largura_e_altura_nos_giros_de_lado() {
        assert_eq!(buffer_size((1080, 1920), Transform::_90), (1920, 1080));
        assert_eq!(buffer_size((1080, 1920), Transform::_270), (1920, 1080));
        assert_eq!(buffer_size((1920, 1080), Transform::_180), (1920, 1080));
        assert_eq!(buffer_size((1920, 1080), Transform::Normal), (1920, 1080));
    }
}
