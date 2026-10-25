#[derive(Debug, Clone)]
pub struct Source {
    pub raw: String,
    pub label: String,
}

impl Source {
    pub fn new(raw: &str) -> Self {
        let label = match raw.strip_prefix("Window: ") {
            Some(window) => format!(
                "Janela: {}",
                window
                    .rsplit_once(" (")
                    .map_or(window, |(title, _)| title)
                    .trim()
            ),
            None => raw.to_owned(),
        };
        let label = label.chars().filter(|c| !c.is_control()).collect();
        Self {
            raw: raw.to_owned(),
            label,
        }
    }
}

pub struct Selection {
    pub sources: Vec<Source>,
    pub query: String,
    pub selected: usize,
}

impl Selection {
    pub fn new(input: &str) -> Self {
        let sources = input
            .lines()
            .filter(|line| !line.is_empty())
            .take(1024)
            .map(Source::new)
            .collect();
        Self {
            sources,
            query: String::new(),
            selected: 0,
        }
    }

    pub fn visible(&self) -> Vec<(usize, &Source)> {
        let query = self.query.to_lowercase();
        self.sources
            .iter()
            .enumerate()
            .filter(|(_, source)| source.label.to_lowercase().contains(&query))
            .collect()
    }

    pub fn chosen(&self) -> Option<&Source> {
        self.visible().get(self.selected).map(|(_, source)| *source)
    }

    pub fn search(&mut self, query: String) {
        self.query = query;
        self.selected = 0;
    }

    pub fn step(&mut self, delta: isize) {
        let count = self.visible().len() as isize;
        if count == 0 {
            return;
        }
        self.selected = (self.selected as isize + delta).rem_euclid(count) as usize;
    }
}
