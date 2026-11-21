use crate::domain::{Activities, Activity, ServiceState, Suspension, Switch, TrainingState};

pub fn training(learning: Switch, state: ServiceState, activities: Activities) -> TrainingState {
    if matches!(
        state,
        ServiceState::Disabled | ServiceState::Starting | ServiceState::Safe
    ) {
        return TrainingState::Unavailable;
    }
    suspension(learning, state, activities).map_or(TrainingState::Allowed, TrainingState::Suspended)
}

fn suspension(learning: Switch, state: ServiceState, activities: Activities) -> Option<Suspension> {
    if learning == Switch::Off {
        return Some(Suspension::UserChoice);
    }
    if state == ServiceState::Degraded {
        return Some(Suspension::Budget);
    }
    if activities.contains(Activity::CriticalPressure) {
        return Some(Suspension::Pressure);
    }
    activities
        .is_sensitive()
        .then_some(Suspension::SensitiveActivity)
}
