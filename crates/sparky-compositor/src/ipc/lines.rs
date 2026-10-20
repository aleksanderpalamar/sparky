pub fn take_lines(pending: &mut Vec<u8>) -> Vec<String> {
    let mut lines = Vec::new();
    while let Some(position) = pending.iter().position(|byte| *byte == b'\n') {
        let line: Vec<u8> = pending.drain(..=position).collect();
        lines.push(String::from_utf8_lossy(&line).trim().to_owned());
    }
    lines.retain(|line| !line.is_empty());
    lines
}

#[cfg(test)]
mod tests {
    use super::take_lines;

    #[test]
    fn separa_as_linhas_completas() {
        let mut pending = b"{\"a\":1}\n{\"b\":2}\n".to_vec();
        assert_eq!(take_lines(&mut pending), vec!["{\"a\":1}", "{\"b\":2}"]);
        assert!(pending.is_empty());
    }

    #[test]
    fn guarda_o_pedaco_sem_quebra_para_depois() {
        let mut pending = b"{\"a\":1}\n{\"b\"".to_vec();
        assert_eq!(take_lines(&mut pending), vec!["{\"a\":1}"]);
        assert_eq!(pending, b"{\"b\"".to_vec());
    }

    #[test]
    fn ignora_linhas_vazias() {
        let mut pending = b"\n\n".to_vec();
        assert!(take_lines(&mut pending).is_empty());
    }
}
