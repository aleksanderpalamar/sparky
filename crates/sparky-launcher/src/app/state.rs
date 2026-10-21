use sparky_theme::Skin;
use sparky_ui::viewport::Viewport;

use crate::domain::app::App;
use crate::domain::search::rank;

pub const VISIBLE: usize = 8;

pub struct Launcher {
    pub apps: Vec<App>,
    pub query: String,
    pub cursor: usize,
    pub list: Viewport,
    pub skin: Skin,
}

impl Launcher {
    pub fn new(apps: Vec<App>, skin: Skin) -> Self {
        Self { apps, query: String::new(), cursor: 0, list: Viewport::new(VISIBLE), skin }
    }

    pub fn results(&self) -> Vec<&App> {
        rank(&self.apps, &self.query)
    }

    pub fn type_query(&mut self, text: String) {
        self.query = text;
        self.cursor = 0;
        self.list = Viewport::new(VISIBLE);
    }

    pub fn step(&mut self, delta: isize) {
        let count = self.results().len();
        self.cursor = self.cursor.saturating_add_signed(delta).min(count.saturating_sub(1));
        self.list.follow(self.cursor, count);
    }

    pub fn chosen(&self) -> Option<App> {
        self.results().get(self.cursor).map(|app| (*app).clone())
    }

    pub fn pick(&self, row: usize) -> Option<App> {
        self.results().get(self.list.top() + row).map(|app| (*app).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{Launcher, VISIBLE};
    use crate::domain::app::{App, Runs};
    use sparky_theme::Skin;

    fn lancador(nomes: &[&str]) -> Launcher {
        let apps = nomes
            .iter()
            .map(|name| App {
                id: name.to_lowercase(),
                name: (*name).to_owned(),
                detail: None,
                keywords: Vec::new(),
                command: name.to_lowercase(),
                runs: Runs::Directly,
            })
            .collect();
        Launcher::new(apps, Skin::default())
    }

    #[test]
    fn digitar_filtra_e_volta_a_selecao_para_o_primeiro() {
        let mut launcher = lancador(&["Firefox", "Fonts", "Kitty"]);
        launcher.step(2);
        launcher.type_query("fo".to_owned());
        assert_eq!(launcher.cursor, 0);
        assert_eq!(launcher.chosen().map(|app| app.name), Some("Fonts".to_owned()));
    }

    #[test]
    fn andar_para_nas_pontas_da_lista() {
        let mut launcher = lancador(&["A", "B", "C"]);
        launcher.step(-1);
        assert_eq!(launcher.cursor, 0);
        launcher.step(9);
        assert_eq!(launcher.chosen().map(|app| app.name), Some("C".to_owned()));
    }

    #[test]
    fn a_lista_rola_para_mostrar_a_selecao_e_o_clique_conta_a_rolagem() {
        let nomes: Vec<String> = (0..12).map(|index| format!("App {index:02}")).collect();
        let referencias: Vec<&str> = nomes.iter().map(String::as_str).collect();
        let mut launcher = lancador(&referencias);
        launcher.step(VISIBLE as isize + 1);
        assert_eq!(launcher.list.top(), 2);
        assert_eq!(launcher.pick(0).map(|app| app.name), Some("App 02".to_owned()));
    }

    #[test]
    fn sem_resultado_nao_ha_escolha() {
        let mut launcher = lancador(&["Firefox"]);
        launcher.type_query("zzz".to_owned());
        assert_eq!(launcher.chosen(), None);
    }
}
