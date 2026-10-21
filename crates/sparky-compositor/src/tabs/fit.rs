const ELLIPSIS: char = '…';

pub fn fit(text: &str, room: f32, measure: impl Fn(&str) -> f32) -> String {
    if measure(text) <= room {
        return text.to_owned();
    }
    let chars: Vec<char> = text.chars().collect();
    (0..chars.len())
        .rev()
        .map(|keep| {
            let mut cut: String = chars[..keep].iter().collect::<String>().trim_end().to_owned();
            cut.push(ELLIPSIS);
            cut
        })
        .find(|candidate| measure(candidate) <= room)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::fit;

    fn sete_por_letra(text: &str) -> f32 {
        text.chars().count() as f32 * 7.0
    }

    #[test]
    fn titulo_que_cabe_fica_inteiro() {
        assert_eq!(fit("kitty", 100.0, sete_por_letra), "kitty");
    }

    #[test]
    fn titulo_longo_e_cortado_com_reticencias() {
        assert_eq!(fit("~/Projetos/sparky", 70.0, sete_por_letra), "~/Projeto…");
    }

    #[test]
    fn o_corte_nao_deixa_espaco_antes_das_reticencias() {
        assert_eq!(fit("abc defgh", 35.0, sete_por_letra), "abc…");
    }

    #[test]
    fn sem_espaco_nem_para_as_reticencias_fica_vazio() {
        assert_eq!(fit("kitty", 3.0, sete_por_letra), "");
    }
}
