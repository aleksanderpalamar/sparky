use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Activity {
    Idle,
    Desktop,
    Development,
    Compiling,
    Gaming,
    Recording,
    Communication,
    CriticalPressure,
    Unknown,
}

pub const ACTIVITIES: [Activity; 9] = [
    Activity::Idle,
    Activity::Desktop,
    Activity::Development,
    Activity::Compiling,
    Activity::Gaming,
    Activity::Recording,
    Activity::Communication,
    Activity::CriticalPressure,
    Activity::Unknown,
];

impl Activity {
    fn bit(self) -> u16 {
        1 << self as u16
    }

    pub fn is_sensitive(self) -> bool {
        matches!(
            self,
            Self::Compiling
                | Self::Gaming
                | Self::Recording
                | Self::Communication
                | Self::CriticalPressure
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Activities(u16);

impl Activities {
    pub fn with(self, activity: Activity) -> Self {
        Self(self.0 | activity.bit())
    }

    pub fn contains(self, activity: Activity) -> bool {
        self.0 & activity.bit() != 0
    }

    pub fn iter(self) -> impl Iterator<Item = Activity> {
        ACTIVITIES
            .into_iter()
            .filter(move |activity| self.contains(*activity))
    }

    pub fn is_sensitive(self) -> bool {
        self.iter().any(Activity::is_sensitive)
    }
}

impl FromIterator<Activity> for Activities {
    fn from_iter<I: IntoIterator<Item = Activity>>(items: I) -> Self {
        items.into_iter().fold(Self::default(), Self::with)
    }
}

impl Serialize for Activities {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter())
    }
}

impl<'de> Deserialize<'de> for Activities {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Vec::<Activity>::deserialize(deserializer)?
            .into_iter()
            .collect())
    }
}
