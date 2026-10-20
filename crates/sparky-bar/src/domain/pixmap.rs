pub fn argb_to_rgba(argb: &[u8]) -> Vec<u8> {
    argb.chunks_exact(4)
        .flat_map(|pixel| [pixel[1], pixel[2], pixel[3], pixel[0]])
        .collect()
}

pub fn best_size(sizes: &[(i32, i32)], wanted: i32) -> Option<usize> {
    let large_enough = sizes
        .iter()
        .enumerate()
        .filter(|(_, (width, _))| *width >= wanted)
        .min_by_key(|(_, (width, _))| *width)
        .map(|(index, _)| index);
    large_enough.or_else(|| {
        sizes
            .iter()
            .enumerate()
            .max_by_key(|(_, (width, _))| *width)
            .map(|(index, _)| index)
    })
}

#[cfg(test)]
mod tests {
    use super::{argb_to_rgba, best_size};

    #[test]
    fn move_o_alfa_do_comeco_para_o_fim() {
        assert_eq!(argb_to_rgba(&[0xff, 0x10, 0x20, 0x30]), vec![0x10, 0x20, 0x30, 0xff]);
    }

    #[test]
    fn ignora_bytes_que_nao_fecham_um_pixel() {
        assert_eq!(argb_to_rgba(&[1, 2, 3, 4, 5, 6]), vec![2, 3, 4, 1]);
    }

    #[test]
    fn escolhe_o_menor_tamanho_que_cabe() {
        assert_eq!(best_size(&[(16, 16), (22, 22), (48, 48)], 20), Some(1));
    }

    #[test]
    fn sem_tamanho_grande_o_bastante_fica_o_maior() {
        assert_eq!(best_size(&[(16, 16), (12, 12)], 22), Some(0));
    }

    #[test]
    fn sem_tamanhos_nao_ha_escolha() {
        assert_eq!(best_size(&[], 22), None);
    }
}
