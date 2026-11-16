use crate::domain::groups::Group;

pub trait GroupStore {
    fn load(&self) -> Result<Vec<Group>, String>;
    fn save(&self, groups: &[Group]) -> Result<(), String>;
}
