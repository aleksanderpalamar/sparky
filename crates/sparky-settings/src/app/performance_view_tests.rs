use iced::Size;

use super::message::Message;
use super::test_support::{Gerenciador, Sparky, painel};
use super::text_probe::{snapshot, visible_texts};
use super::update::update;
use crate::domain::section::Section;

const WINDOW: Size = Size {
    width: 760.0,
    height: 860.0,
};

fn aberto() -> super::state::Settings {
    let (mut settings, _) = painel(Some(""), Sparky::default());
    settings.services.resources = Box::new(Gerenciador::default());
    let _ = update(&mut settings, Message::Section(Section::Performance));
    settings
}

fn contem(textos: &[String], esperados: &[&str]) {
    for esperado in esperados {
        assert!(
            textos.iter().any(|texto| texto == esperado),
            "faltou {esperado}: {textos:?}"
        );
    }
}

#[test]
fn cartoes_mostram_gerenciador_modo_inteligencia_e_situacao() {
    let settings = aberto();
    snapshot(
        crate::view::view(&settings),
        settings.context.skin,
        WINDOW,
        "desempenho",
    );
    let textos = visible_texts(crate::view::performance::view(&settings));
    contem(
        &textos,
        &[
            "Desempenho",
            "Gerenciador adaptativo de recursos",
            "Modo de operação",
            "Só mede e simula, sem mexer em nada.",
            "Inteligência",
            "Aprendizado contínuo",
            "Otimização em jogos",
            "Proteção de aplicativos ativos",
            "Notificações de otimização",
            "obs",
            "Configurações avançadas",
            "Serviço adaptativo ativo · Observando",
        ],
    );
    assert!(!textos.iter().any(|texto| texto == "Limite de memória"));
}

#[test]
fn configuracoes_avancadas_mostram_so_fatos_do_servico() {
    let mut settings = aberto();
    let _ = update(&mut settings, Message::PerformanceAdvanced);
    snapshot(
        crate::view::view(&settings),
        settings.context.skin,
        WINDOW,
        "desempenho-avancado",
    );
    let textos = visible_texts(crate::view::performance::view(&settings));
    contem(
        &textos,
        &[
            "Limite de memória",
            "64 MiB",
            "Não usa, só CPU",
            "GRU · 6792 parâmetros",
            "Ativada",
            "Somente local",
            "Apagar o que foi aprendido",
        ],
    );
}
