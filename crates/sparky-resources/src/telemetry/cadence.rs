use std::time::Duration;

use crate::domain::{Activities, Activity};

const IDLE: Duration = Duration::from_secs(5);
const ACTIVE: Duration = Duration::from_secs(2);
const DEVELOPMENT: Duration = Duration::from_millis(1500);
const CRITICAL: Duration = Duration::from_millis(250);

pub fn interval(activities: Activities) -> Duration {
    if activities.contains(Activity::CriticalPressure) {
        return CRITICAL;
    }
    if activities.is_sensitive() {
        return ACTIVE;
    }
    if activities.contains(Activity::Development) {
        return DEVELOPMENT;
    }
    if activities.contains(Activity::Desktop) {
        return ACTIVE;
    }
    IDLE
}
