pub fn flip_rows(bytes: &[u8], stride: usize) -> Vec<u8> {
    if stride == 0 {
        return bytes.to_vec();
    }
    bytes.chunks_exact(stride).rev().flatten().copied().collect()
}

#[cfg(test)]
mod tests {
    use super::flip_rows;

    #[test]
    fn a_ultima_linha_vira_a_primeira() {
        assert_eq!(flip_rows(&[1, 2, 3, 4, 5, 6], 2), vec![5, 6, 3, 4, 1, 2]);
    }

    #[test]
    fn uma_linha_so_nao_muda() {
        assert_eq!(flip_rows(&[1, 2, 3], 3), vec![1, 2, 3]);
    }

    #[test]
    fn largura_zero_nao_quebra() {
        assert_eq!(flip_rows(&[1, 2], 0), vec![1, 2]);
    }
}
