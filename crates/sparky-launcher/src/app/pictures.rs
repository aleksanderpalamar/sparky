use std::collections::HashMap;
use std::path::PathBuf;

use iced::widget::image::Handle;

use crate::domain::icon::Picture;

#[derive(Debug, Clone)]
pub enum Shown {
    Vector(PathBuf),
    Raster(Handle),
}

pub fn shown(pictures: HashMap<String, Picture>) -> HashMap<String, Shown> {
    pictures
        .into_iter()
        .map(|(id, picture)| (id, prepared(picture)))
        .collect()
}

fn prepared(picture: Picture) -> Shown {
    match picture {
        Picture::Vector(path) => Shown::Vector(path),
        Picture::Raster(raster) => {
            Shown::Raster(Handle::from_rgba(raster.width, raster.height, raster.rgba))
        }
    }
}
