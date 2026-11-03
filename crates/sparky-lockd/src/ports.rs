use std::future::Future;
use std::os::fd::OwnedFd;

use crate::domain::guard::Screen;

pub trait Logind {
    fn inhibit(&self) -> impl Future<Output = Result<OwnedFd, String>> + Send;
    fn hint(&self, screen: Screen) -> impl Future<Output = Result<(), String>> + Send;
}

pub trait Compositor {
    fn lock(&self) -> Result<(), String>;
    fn unlock(&self) -> Result<(), String>;
}
