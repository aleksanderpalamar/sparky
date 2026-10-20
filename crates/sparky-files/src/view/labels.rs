use crate::domain::entry::{Entry, Kind};
use crate::domain::listing::Listing;
use crate::domain::size::human;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Directory,
    File,
    Hidden,
    Marked,
}

pub fn tone(listing: &Listing, entry: &Entry) -> Tone {
    match entry.kind {
        _ if listing.is_marked(entry) => Tone::Marked,
        _ if entry.hidden() => Tone::Hidden,
        Kind::Directory => Tone::Directory,
        Kind::File => Tone::File,
    }
}

pub fn name(entry: &Entry, tone: Tone) -> String {
    let mark = match tone {
        Tone::Marked => "● ",
        Tone::Directory | Tone::File | Tone::Hidden => "",
    };
    let slash = match entry.kind {
        Kind::Directory => "/",
        Kind::File => "",
    };
    format!("{mark}{}{slash}", entry.name)
}

pub fn size(entry: &Entry) -> String {
    match entry.kind {
        Kind::Directory => String::new(),
        Kind::File => human(entry.size),
    }
}

pub fn date(entry: &Entry) -> String {
    entry
        .modified
        .map(|moment| moment.format("%d/%m/%Y %H:%M").to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{Tone, date, name, size, tone};
    use crate::domain::entry::{Entry, Kind, entry};
    use crate::domain::listing::{Hidden, Listing};
    use chrono::NaiveDate;

    #[test]
    fn pasta_ganha_barra_e_item_marcado_ganha_bolinha() {
        assert_eq!(name(&entry("Fotos", Kind::Directory), Tone::Directory), "Fotos/");
        assert_eq!(name(&entry("a.txt", Kind::File), Tone::Marked), "● a.txt");
    }

    #[test]
    fn o_tom_segue_marca_oculto_e_tipo() {
        let mut listing = Listing::new(vec![entry("a.txt", Kind::File), entry(".b", Kind::File)], Hidden::Shown);
        listing.select_name("a.txt");
        listing.toggle_mark();
        assert_eq!(tone(&listing, &entry("a.txt", Kind::File)), Tone::Marked);
        assert_eq!(tone(&listing, &entry(".b", Kind::File)), Tone::Hidden);
        assert_eq!(tone(&listing, &entry("Docs", Kind::Directory)), Tone::Directory);
    }

    #[test]
    fn pasta_nao_mostra_tamanho_e_arquivo_mostra() {
        let arquivo = Entry { size: 1536, ..entry("a.txt", Kind::File) };
        assert_eq!(size(&arquivo), "1,5 kB");
        assert_eq!(size(&entry("Docs", Kind::Directory)), "");
    }

    #[test]
    fn a_data_fica_no_formato_brasileiro() {
        let modified = NaiveDate::from_ymd_opt(2026, 10, 20).and_then(|d| d.and_hms_opt(16, 4, 0));
        assert_eq!(date(&Entry { modified, ..entry("a", Kind::File) }), "20/10/2026 16:04");
        assert_eq!(date(&entry("a", Kind::File)), "");
    }
}
