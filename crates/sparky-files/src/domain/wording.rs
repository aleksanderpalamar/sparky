use crate::domain::clip::Clip;
use crate::i18n;

pub fn items(count: usize) -> String {
    match count {
        1 => i18n::text("one-item"),
        other => i18n::with("many-items", "count", other.to_string()),
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
        let key = if marked == 1 {
            "one-marked"
        } else {
            "many-marked"
        };
        parts.push(i18n::with(key, "count", marked.to_string()));
    }
    if let Some(clip) = clip {
        let count = clip.paths().len();
        let key = match (clip, count) {
            (Clip::Cut(_), 1) => "one-cut",
            (Clip::Cut(_), _) => "many-cut",
            (Clip::Copy(_), 1) => "one-copied",
            (Clip::Copy(_), _) => "many-copied",
        };
        parts.push(i18n::with(key, "count", count.to_string()));
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
        assert_eq!(
            summary(12, 1, Some(&clip)),
            "12 itens · 1 marcado · 2 recortados"
        );
    }
}
