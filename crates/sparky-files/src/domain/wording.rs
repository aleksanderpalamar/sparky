use crate::domain::clip::Clip;

pub fn items(count: usize) -> String {
    match count {
        1 => "1 item".to_owned(),
        other => format!("{other} itens"),
    }
}

pub fn agreed(word: &str, count: usize) -> String {
    match count {
        1 => word.to_owned(),
        _ => format!("{word}s"),
    }
}

pub fn summary(len: usize, marked: usize, clip: Option<&Clip>) -> String {
    let mut parts = vec![items(len)];
    if marked > 0 {
        parts.push(format!("{marked} {}", agreed("marcado", marked)));
    }
    if let Some(clip) = clip {
        let count = clip.paths().len();
        parts.push(format!("{count} {}", agreed(clip.verb(), count)));
    }
    parts.join(" · ")
}

#[cfg(test)]
mod tests {
    use super::{agreed, items, summary};
    use crate::domain::clip::Clip;
    use std::path::PathBuf;

    #[test]
    fn um_item_fica_no_singular() {
        assert_eq!(items(1), "1 item");
        assert_eq!(items(0), "0 itens");
        assert_eq!(agreed("colado", 2), "colados");
    }

    #[test]
    fn o_resumo_junta_marcas_e_area_de_transferencia() {
        let clip = Clip::Cut(vec![PathBuf::from("/a"), PathBuf::from("/b")]);
        assert_eq!(summary(12, 0, None), "12 itens");
        assert_eq!(summary(12, 1, Some(&clip)), "12 itens · 1 marcado · 2 recortados");
    }
}
