use std::future::Future;

use crate::domain::guard::Screen;
use crate::domain::inhibitions::Hold;

pub trait Logind {
    type Inhibitor: Send;

    fn inhibit(&self) -> impl Future<Output = Result<Self::Inhibitor, String>> + Send;
    fn hint(&self, screen: Screen) -> impl Future<Output = Result<(), String>> + Send;
}

pub trait Compositor {
    fn lock(&self) -> Result<(), String>;
    fn unlock(&self) -> Result<(), String>;
    fn idle(&self, hold: Hold) -> Result<(), String>;
}
