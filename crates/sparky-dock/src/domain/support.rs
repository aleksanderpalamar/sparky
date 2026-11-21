use super::app::{App, Launch};

pub fn app(id: &str, name: &str) -> App {
    App {
        id: id.to_owned(),
        name: name.to_owned(),
        icon: None,
        launch: Launch {
            arguments: vec![id.to_owned()],
            terminal: false,
        },
        window_class: None,
    }
}

pub fn ids(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_owned()).collect()
}
