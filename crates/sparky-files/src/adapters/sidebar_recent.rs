use std::fs;
use std::path::Path;

use gio::prelude::*;
use quick_xml::Reader;
use quick_xml::events::Event;

use super::sidebar_source::item;
use crate::domain::sidebar_location::SidebarItem;

pub fn read_recent(data: &Path) -> Result<Vec<SidebarItem>, String> {
    let contents = match fs::read_to_string(data.join("recently-used.xbel")) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut reader = Reader::from_str(&contents);
    let mut recent = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(tag) | Event::Empty(tag)) if tag.name().as_ref() == b"bookmark" => {
                let attributes: Vec<_> = tag
                    .attributes()
                    .flatten()
                    .filter_map(|attribute| {
                        let value = attribute
                            .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                            .ok()?
                            .into_owned();
                        Some((attribute.key.as_ref().to_vec(), value))
                    })
                    .collect();
                let href = attributes
                    .iter()
                    .find(|(key, _)| key == b"href")
                    .map(|(_, value)| value);
                let visited = attributes
                    .iter()
                    .find(|(key, _)| key == b"visited")
                    .map_or("", |(_, value)| value.as_str());
                if let Some(path) = href
                    .and_then(|uri| gio::File::for_uri(uri).path())
                    .filter(|path| path.is_file())
                {
                    recent.push((visited.to_owned(), path));
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(error.to_string()),
            _ => {}
        }
    }
    recent.sort_by(|left, right| right.0.cmp(&left.0));
    Ok(recent
        .into_iter()
        .take(100)
        .map(|(_, path)| item(path))
        .collect())
}
