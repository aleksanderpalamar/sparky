use std::path::PathBuf;

pub const ICON_PIXELS: u32 = 28;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Icon {
    Named(String),
    Symbolic(&'static [&'static str]),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Picture {
    Vector(PathBuf),
    Raster(Raster),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raster {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
