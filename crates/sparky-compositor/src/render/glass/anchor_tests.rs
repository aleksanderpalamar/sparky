use smithay::backend::renderer::element::Id;

use super::anchor::anchors;

#[test]
fn o_vidro_ancora_pela_identidade_original_da_superficie() {
    let barra = Id::new();
    let app = Id::new();
    let fundo = Id::new();
    let ids = [app.clone(), barra.clone(), fundo];
    let ancoras = anchors(ids.iter(), vec![(barra, "barra"), (app, "app")]);
    assert_eq!(ancoras, vec![(0, "app"), (1, "barra")]);
}

#[test]
fn superficie_sem_elemento_fica_sem_vidro() {
    let ids = [Id::new()];
    assert!(anchors(ids.iter(), vec![(Id::new(), "sumiu")]).is_empty());
}
