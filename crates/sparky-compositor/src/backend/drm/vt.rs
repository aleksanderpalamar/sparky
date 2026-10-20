const FIRST_SWITCH: u32 = 0x1008_fe01;
const LAST_SWITCH: u32 = 0x1008_fe0c;

pub fn terminal_for(keysym: u32) -> Option<i32> {
    (FIRST_SWITCH..=LAST_SWITCH)
        .contains(&keysym)
        .then(|| (keysym - FIRST_SWITCH + 1) as i32)
}

#[cfg(test)]
mod tests {
    use super::terminal_for;

    #[test]
    fn as_teclas_de_troca_vao_do_terminal_um_ao_doze() {
        assert_eq!(terminal_for(0x1008_fe01), Some(1));
        assert_eq!(terminal_for(0x1008_fe0c), Some(12));
    }

    #[test]
    fn outras_teclas_nao_trocam_de_terminal() {
        assert_eq!(terminal_for(0x1008_fe00), None);
        assert_eq!(terminal_for(0x1008_fe0d), None);
        assert_eq!(terminal_for(0xff0d), None);
    }
}
