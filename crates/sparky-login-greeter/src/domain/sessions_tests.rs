use super::sessions::{Session, SessionFile, Sessions, sessions};

fn arquivo(id: &str, contents: &str) -> SessionFile {
    SessionFile {
        id: id.to_owned(),
        contents: contents.to_owned(),
    }
}

fn entrada(name: &str, exec: &str) -> String {
    format!("[Desktop Entry]\nName={name}\nExec={exec}\nType=Application\n")
}

fn tudo_instalado(_: &str) -> bool {
    true
}

fn sessao(id: &str, name: &str) -> Session {
    Session {
        id: id.to_owned(),
        name: name.to_owned(),
    }
}

fn lista() -> Vec<Session> {
    vec![sessao("hyprland", "Hyprland"), sessao("sparky", "Sparky")]
}

#[test]
fn lista_as_sessoes_wayland_pelo_nome() {
    let files = vec![
        arquivo("sparky", &entrada("Sparky", "sparky")),
        arquivo("hyprland", &entrada("Hyprland", "Hyprland")),
    ];
    assert_eq!(sessions(files, "pt_BR", tudo_instalado), lista());
}

#[test]
fn escondidas_sem_exec_e_nao_instaladas_ficam_de_fora() {
    let files = vec![
        arquivo(
            "oculta",
            "[Desktop Entry]\nName=Oculta\nExec=oculta\nHidden=true\n",
        ),
        arquivo(
            "sem-menu",
            "[Desktop Entry]\nName=Sem menu\nExec=sem-menu\nNoDisplay=true\n",
        ),
        arquivo("sem-exec", "[Desktop Entry]\nName=Sem exec\n"),
        arquivo(
            "ausente",
            "[Desktop Entry]\nName=Ausente\nExec=ausente\nTryExec=ausente\n",
        ),
        arquivo("sparky", &entrada("Sparky", "sparky")),
    ];
    let instalado = |program: &str| program != "ausente";
    assert_eq!(
        sessions(files, "pt_BR", instalado),
        vec![sessao("sparky", "Sparky")]
    );
}

#[test]
fn quando_o_arquivo_se_repete_vale_o_da_primeira_pasta() {
    let files = vec![
        arquivo("sparky", &entrada("Sparky local", "sparky")),
        arquivo("sparky", &entrada("Sparky", "sparky")),
    ];
    assert_eq!(
        sessions(files, "C", tudo_instalado),
        vec![sessao("sparky", "Sparky local")]
    );
}

#[test]
fn o_nome_segue_o_idioma_do_sistema() {
    let files = vec![arquivo(
        "sparky",
        "[Desktop Entry]\nName=Sparky\nName[pt_BR]=Sparky BR\nExec=sparky\n",
    )];
    assert_eq!(
        sessions(files, "pt_BR", tudo_instalado),
        vec![sessao("sparky", "Sparky BR")]
    );
}

#[test]
fn a_sessao_escolhida_e_a_preferida_e_sem_ela_o_sparky() {
    let preferida = Sessions::new(lista(), Some("hyprland"));
    assert_eq!(
        preferida
            .map(|sessions| sessions.current().id.clone())
            .as_deref(),
        Some("hyprland")
    );
    let padrao = Sessions::new(lista(), None);
    assert_eq!(
        padrao
            .map(|sessions| sessions.current().id.clone())
            .as_deref(),
        Some("sparky")
    );
    let desconhecida = Sessions::new(lista(), Some("gnome"));
    assert_eq!(
        desconhecida
            .map(|sessions| sessions.current().id.clone())
            .as_deref(),
        Some("sparky")
    );
}

#[test]
fn sem_o_sparky_instalado_vale_a_primeira_da_lista() {
    let escolhida = Sessions::new(
        vec![sessao("hyprland", "Hyprland"), sessao("niri", "Niri")],
        None,
    );
    assert_eq!(
        escolhida
            .map(|sessions| sessions.current().id.clone())
            .as_deref(),
        Some("hyprland")
    );
}

#[test]
fn sem_nenhuma_sessao_nao_ha_o_que_escolher() {
    assert!(Sessions::new(Vec::new(), None).is_none());
}

#[test]
fn trocar_de_sessao_da_a_volta_na_lista() {
    let Some(mut sessions) = Sessions::new(lista(), None) else {
        panic!("a lista tem sessões");
    };
    sessions.next();
    assert_eq!(sessions.current().id, "hyprland");
    sessions.next();
    assert_eq!(sessions.current().id, "sparky");
}

#[test]
fn o_rotulo_e_o_da_foto() {
    let Some(sessions) = Sessions::new(lista(), None) else {
        panic!("a lista tem sessões");
    };
    assert_eq!(sessions.label(), "Session (Sparky)");
}
