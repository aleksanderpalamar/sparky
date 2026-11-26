use sparky_resources::application::replay::shadow::{Mode, Scenario, replay};

fn main() {
    println!(
        "cenario,modo,intervencoes,latencia_s,falsos_positivos,encerrados,escritas_em_protegido"
    );
    for (name, scenario) in scenarios() {
        for mode in [Mode::Observed, Mode::Confirmed] {
            let outcome = replay(&scenario, mode);
            println!(
                "{name},{},{},{},{},{},{}",
                label(mode),
                outcome.interventions,
                outcome
                    .latency
                    .map_or_else(|| "-".to_owned(), |latency| latency.to_string()),
                outcome.false_positives,
                outcome.finished,
                outcome.protected_writes
            );
        }
    }
    println!(
        "observacao: no fluxo atual a previsao neural nao entra na decisao; com ou sem ativo aprovado o modo atual decide igual"
    );
}

fn label(mode: Mode) -> &'static str {
    match mode {
        Mode::Observed => "atual",
        Mode::Confirmed => "sombra",
    }
}

fn scenarios() -> Vec<(&'static str, Scenario)> {
    let shaped =
        |ticks: usize, shape: fn(usize) -> f32, transient: bool, protected_hog: bool| Scenario {
            stalls: (0..ticks).map(shape).collect(),
            transient,
            protected_hog,
        };
    vec![
        ("sustentada", shaped(60, |_| 40.0, false, false)),
        (
            "passageira",
            shaped(60, |tick| if tick < 2 { 40.0 } else { 5.0 }, true, false),
        ),
        ("rampa", shaped(60, |tick| 5.0 + tick as f32, false, false)),
        (
            "oscilante",
            shaped(
                60,
                |tick| if tick % 6 < 2 { 35.0 } else { 8.0 },
                true,
                false,
            ),
        ),
        ("calma", shaped(60, |_| 5.0, false, false)),
        ("jogo_protegido", shaped(60, |_| 40.0, false, true)),
    ]
}
