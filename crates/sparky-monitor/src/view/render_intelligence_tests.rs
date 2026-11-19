use std::sync::Arc;

use sparky_resources::application::{Footprint, Insight, Outcome, Progress, Round};
use sparky_resources::domain::{
    Activities, Activity, Change, Metric, Mode, ServiceState, Switch, TrainingState,
};
use sparky_resources::orchestration::{Decision, Purpose, Verdict};
use sparky_resources::protocol::Status;
use sparky_resources::safety::Outcome as Restoration;

use super::render_tests::{contem, renderizar};
use crate::app::message::Message;
use crate::app::page::Page;
use crate::app::test_support::{Sinais, monitor};
use crate::app::update::update;
use crate::domain::decision::Lens;
use crate::domain::intelligence::{Advice, Briefing};

fn round(at_seconds: u64, neural: f64, statistical: f64) -> Round {
    Round {
        at_seconds,
        outcome: Outcome::BehindStatistics,
        neural: Some(neural),
        statistical,
    }
}

fn briefing() -> Briefing {
    let status = Status {
        service: "0.32.0".to_owned(),
        state: ServiceState::Adaptive,
        enabled: Switch::On,
        mode: Mode::Adaptive,
        learning: Switch::On,
        training: TrainingState::Allowed,
        activities: Activities::default()
            .with(Activity::Gaming)
            .with(Activity::Recording),
        progress: Progress {
            collected: 312,
            needed: 600,
            window: 7200,
            epoch: 3,
            epochs: 20,
            rounds: 2,
            parameters: 6792,
            history: vec![round(30, 5.2, 4.0), round(90, 4.1, 3.8)],
        },
        footprint: Some(Footprint {
            cpu_percent: 0.04,
            rss_kib: 3 * 1024,
            limit_kib: Some(64 * 1024),
        }),
        guarded: vec!["obs".to_owned()],
        adjusting: true,
        notifications: Switch::On,
    };
    let insight = Insight {
        metric: Metric::GpuBusy,
        current: Some(37.0),
        estimate_short: None,
        estimate_long: None,
        anomaly: None,
        neural_short: None,
        neural_long: None,
    };
    let decision = Decision {
        at_seconds: 179,
        activities: Activities::default().with(Activity::Gaming),
        purpose: Purpose::EaseCpu,
        process: "steamwebhelper".to_owned(),
        change: Change::Affinity(vec![5, 11]),
        stall: Some(24.0),
        verdict: Verdict::Reverted(Restoration::Restored),
    };
    Briefing {
        status,
        metrics: vec![insight],
        decisions: vec![decision],
    }
}

#[test]
fn servico_fora_do_ar_e_avisado() {
    let mut monitor = monitor(Arc::new(Sinais::default()));
    let _ = update(&mut monitor, Message::Open(Page::Intelligence));
    monitor.advice = Some(Advice::Unavailable);
    let textos = renderizar(&monitor, "inteligencia-fora");
    contem(
        &textos,
        &["O gerenciador de recursos não está respondendo."],
    );
}

#[test]
fn as_tres_abas_mostram_dados_reais_do_servico() {
    let mut monitor = monitor(Arc::new(Sinais::default()));
    let _ = update(&mut monitor, Message::Open(Page::Intelligence));
    monitor.advice = Some(Advice::Ready(Box::new(briefing())));
    let visao = renderizar(&monitor, "inteligencia-visao");
    contem(
        &visao,
        &[
            "Visão geral",
            "Jogo + Gravação",
            "Adaptativo",
            "0.04%",
            "3,00 MiB / 64,00 MiB",
            "Ajustes revertidos",
            "obs",
        ],
    );
    let _ = update(&mut monitor, Message::Look(Lens::Decisions));
    let decisoes = renderizar(&monitor, "inteligencia-decisoes");
    contem(
        &decisoes,
        &[
            "Afinidade de CPU ajustada",
            "steamwebhelper · CPU 5, 11 · aliviar CPU · 00:02:59",
            "revertido",
        ],
    );
    let _ = update(&mut monitor, Message::Look(Lens::Learning));
    let aprendizado = renderizar(&monitor, "inteligencia-aprendizado");
    contem(
        &aprendizado,
        &[
            "GRU",
            "6792",
            "312 / 7200 · mín. 600",
            "00:01:30 · descartado, pior que a estatística",
            "4.10 / 3.80",
            "37.0%",
        ],
    );
}
