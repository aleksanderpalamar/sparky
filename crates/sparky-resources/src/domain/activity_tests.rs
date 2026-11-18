use super::activity::{ACTIVITIES, Activities, Activity};

#[test]
fn reconhece_atividades_simultaneas() {
    let activities: Activities = [
        Activity::Gaming,
        Activity::Recording,
        Activity::Communication,
    ]
    .into_iter()
    .collect();
    assert!(activities.contains(Activity::Gaming));
    assert!(activities.contains(Activity::Recording));
    assert!(!activities.contains(Activity::Desktop));
    assert_eq!(activities.iter().count(), 3);
}

#[test]
fn jogo_gravacao_compilacao_e_pressao_sao_sensiveis() {
    for activity in [
        Activity::Gaming,
        Activity::Recording,
        Activity::Compiling,
        Activity::CriticalPressure,
    ] {
        assert!(activity.is_sensitive(), "{activity:?}");
    }
    for activity in [
        Activity::Idle,
        Activity::Desktop,
        Activity::Development,
        Activity::Communication,
        Activity::Unknown,
    ] {
        assert!(!activity.is_sensitive(), "{activity:?}");
    }
}

#[test]
fn basta_uma_atividade_sensivel_no_conjunto() {
    let calm: Activities = [Activity::Desktop, Activity::Development]
        .into_iter()
        .collect();
    assert!(!calm.is_sensitive());
    assert!(calm.with(Activity::Recording).is_sensitive());
}

#[test]
fn cada_atividade_tem_um_bit_proprio() {
    let all: Activities = ACTIVITIES.into_iter().collect();
    assert_eq!(all.iter().collect::<Vec<_>>(), ACTIVITIES.to_vec());
}

#[test]
fn o_conjunto_vira_lista_de_nomes_e_volta() {
    let activities: Activities = [Activity::Gaming, Activity::Recording]
        .into_iter()
        .collect();
    let json = serde_json::to_string(&activities).unwrap_or_default();
    assert_eq!(json, r#"["gaming","recording"]"#);
    assert_eq!(
        serde_json::from_str::<Activities>(&json).ok(),
        Some(activities)
    );
}
