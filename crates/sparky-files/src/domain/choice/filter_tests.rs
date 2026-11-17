use super::{Filter, Rule};

fn filtro(rules: Vec<Rule>) -> Filter {
    Filter {
        name: "Teste".to_owned(),
        rules,
    }
}

fn sem_mime(_: &str) -> String {
    "application/octet-stream".to_owned()
}

#[test]
fn padroes_com_asterisco_interrogacao_e_classes_ignoram_maiusculas() {
    let videos = filtro(vec![
        Rule::Glob("*.mp4".to_owned()),
        Rule::Glob("*.[mM][kK][vV]".to_owned()),
    ]);
    assert!(videos.accepts("demo-sparky.MP4", sem_mime));
    assert!(videos.accepts("filme.mkv", sem_mime));
    assert!(!videos.accepts("notas.txt", sem_mime));
    let curto = filtro(vec![Rule::Glob("?.txt".to_owned())]);
    assert!(curto.accepts("a.txt", sem_mime));
    assert!(!curto.accepts("ab.txt", sem_mime));
}

#[test]
fn tipo_mime_aceita_exato_e_familia() {
    let imagens = filtro(vec![Rule::Mime("image/*".to_owned())]);
    let png = |_: &str| "image/png".to_owned();
    assert!(imagens.accepts("foto.png", png));
    assert!(!imagens.accepts("notas.txt", |_| "text/plain".to_owned()));
    let pdf = filtro(vec![Rule::Mime("application/pdf".to_owned())]);
    assert!(pdf.accepts("livro.pdf", |_| "application/pdf".to_owned()));
    assert!(!pdf.accepts("livro.epub", |_| "application/epub+zip".to_owned()));
}

#[test]
fn colchete_sem_fechamento_vale_como_letra() {
    let estranho = filtro(vec![Rule::Glob("[a.txt".to_owned())]);
    assert!(estranho.accepts("[a.txt", sem_mime));
    assert!(!estranho.accepts("a.txt", sem_mime));
}
