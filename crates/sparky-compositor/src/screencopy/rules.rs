use smithay::reexports::wayland_server::protocol::wl_shm::Format;
use smithay::utils::Transform;
use sparky_tiling::Rect;

const BYTES_PER_PIXEL: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape {
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub format: Format,
}

pub fn region(output: (i32, i32), requested: Option<Rect>) -> Option<Rect> {
    let whole = Rect::new(0, 0, output.0, output.1);
    match requested {
        None => (whole.width > 0 && whole.height > 0).then_some(whole),
        Some(requested) => whole.intersection(&requested),
    }
}

pub fn accepts(shape: Shape, size: (i32, i32)) -> bool {
    let format = matches!(shape.format, Format::Argb8888 | Format::Xrgb8888);
    format && (shape.width, shape.height) == size && shape.stride >= size.0 * BYTES_PER_PIXEL as i32
}

pub fn copy_bgra(rgba: &[u8], width: usize, destination: &mut [u8], stride: usize) -> bool {
    let row = width * BYTES_PER_PIXEL;
    if width == 0 || stride < row {
        return false;
    }
    let height = rgba.len() / row;
    if destination.len() < stride * height.saturating_sub(1) + row {
        return false;
    }
    for (source, target) in rgba.chunks_exact(row).zip(destination.chunks_mut(stride)) {
        for (pixel, out) in source.chunks_exact(BYTES_PER_PIXEL).zip(target.chunks_exact_mut(BYTES_PER_PIXEL)) {
            out.copy_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }
    }
    true
}

pub fn orient(rgba: &[u8], width: usize, transform: Transform) -> Option<Vec<u8>> {
    let row = width * BYTES_PER_PIXEL;
    if row == 0 {
        return None;
    }
    let (flip_rows, flip_columns) = match transform {
        Transform::Normal => (false, false),
        Transform::Flipped => (false, true),
        Transform::Flipped180 => (true, false),
        Transform::_180 => (true, true),
        _ => return None,
    };
    let rows: Vec<&[u8]> = match flip_rows {
        true => rgba.chunks_exact(row).rev().collect(),
        false => rgba.chunks_exact(row).collect(),
    };
    Some(rows.into_iter().flat_map(|line| mirrored(line, flip_columns)).collect())
}

fn mirrored(line: &[u8], flip: bool) -> Vec<u8> {
    match flip {
        true => line.chunks_exact(BYTES_PER_PIXEL).rev().flatten().copied().collect(),
        false => line.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Shape, accepts, copy_bgra, region};
    use smithay::reexports::wayland_server::protocol::wl_shm::Format;
    use sparky_tiling::Rect;

    #[test]
    fn sem_regiao_vale_o_monitor_inteiro() {
        assert_eq!(region((1920, 1080), None), Some(Rect::new(0, 0, 1920, 1080)));
    }

    #[test]
    fn a_regiao_e_cortada_na_borda_do_monitor() {
        assert_eq!(region((1920, 1080), Some(Rect::new(1800, 1000, 400, 400))), Some(Rect::new(1800, 1000, 120, 80)));
        assert_eq!(region((1920, 1080), Some(Rect::new(5000, 0, 10, 10))), None);
    }

    #[test]
    fn so_aceita_buffer_do_tamanho_e_formato_anunciados() {
        let certo = Shape { width: 100, height: 50, stride: 400, format: Format::Argb8888 };
        assert!(accepts(certo, (100, 50)));
        assert!(accepts(Shape { stride: 512, format: Format::Xrgb8888, ..certo }, (100, 50)));
        assert!(!accepts(Shape { width: 99, ..certo }, (100, 50)));
        assert!(!accepts(Shape { stride: 396, ..certo }, (100, 50)));
        assert!(!accepts(Shape { format: Format::Rgb565, ..certo }, (100, 50)));
    }

    #[test]
    fn copia_trocando_vermelho_com_azul_e_respeitando_a_linha() {
        let rgba = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let mut destino = [0_u8; 20];
        assert!(copy_bgra(&rgba, 2, &mut destino, 12));
        assert_eq!(destino, [3, 2, 1, 4, 7, 6, 5, 8, 0, 0, 0, 0, 11, 10, 9, 12, 15, 14, 13, 16]);
    }

    #[test]
    fn destino_pequeno_demais_e_recusado() {
        let mut destino = [0_u8; 7];
        assert!(!copy_bgra(&[0; 8], 2, &mut destino, 8));
    }
}

#[cfg(test)]
mod orientation_tests {
    use super::orient;
    use smithay::utils::Transform;

    const IMAGEM: [u8; 16] = [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];

    #[test]
    fn monitor_normal_recebe_a_imagem_como_esta() {
        assert_eq!(orient(&IMAGEM, 2, Transform::Normal), Some(IMAGEM.to_vec()));
    }

    #[test]
    fn monitor_espelhado_na_vertical_recebe_as_linhas_invertidas() {
        assert_eq!(orient(&IMAGEM, 2, Transform::Flipped180), Some(vec![3, 3, 3, 3, 4, 4, 4, 4, 1, 1, 1, 1, 2, 2, 2, 2]));
    }

    #[test]
    fn monitor_espelhado_na_horizontal_e_girado_meia_volta() {
        assert_eq!(orient(&IMAGEM, 2, Transform::Flipped), Some(vec![2, 2, 2, 2, 1, 1, 1, 1, 4, 4, 4, 4, 3, 3, 3, 3]));
        assert_eq!(orient(&IMAGEM, 2, Transform::_180), Some(vec![4, 4, 4, 4, 3, 3, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1]));
    }

    #[test]
    fn monitor_girado_de_lado_nao_e_suportado() {
        assert_eq!(orient(&IMAGEM, 2, Transform::_90), None);
    }
}
