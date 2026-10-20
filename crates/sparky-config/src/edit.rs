use toml_edit::{Array, DocumentMut, Item, Table, TableLike, value};

use crate::load::ConfigError;

#[derive(Debug, Clone, Default)]
pub struct Editor {
    document: DocumentMut,
}

impl Editor {
    pub fn parse(contents: &str) -> Result<Self, ConfigError> {
        contents
            .parse::<DocumentMut>()
            .map(|document| Self { document })
            .map_err(|error| ConfigError::Invalid(error.message().to_owned()))
    }

    pub fn set_number(&mut self, table: &str, key: &str, number: i32) {
        if let Some(table) = self.table(table) {
            put(table, key, value(i64::from(number)));
        }
    }

    pub fn set_flag(&mut self, table: &str, key: &str, flag: bool) {
        if let Some(table) = self.table(table) {
            put(table, key, value(flag));
        }
    }

    pub fn set_text(&mut self, table: &str, key: &str, text: Option<&str>) {
        let Some(table) = self.table(table) else {
            return;
        };
        match text.map(str::trim).filter(|text| !text.is_empty()) {
            Some(text) => put(table, key, value(text)),
            None => {
                table.remove(key);
            }
        }
    }

    pub fn set_list(&mut self, key: &str, items: &[String]) {
        let array: Array = items.iter().map(String::as_str).collect();
        put(self.document.as_table_mut(), key, value(array));
    }

    pub fn contents(&self) -> String {
        self.document.to_string()
    }

    fn table(&mut self, name: &str) -> Option<&mut dyn TableLike> {
        if !self.document.get(name).is_some_and(Item::is_table_like) {
            self.document.insert(name, Item::Table(Table::new()));
        }
        self.document.get_mut(name).and_then(Item::as_table_like_mut)
    }
}

fn put(table: &mut dyn TableLike, key: &str, new: Item) {
    let Some(item) = table.get_mut(key) else {
        table.insert(key, new);
        return;
    };
    let decor = item.as_value().map(|old| old.decor().clone());
    *item = new;
    if let (Some(decor), Some(fresh)) = (decor, item.as_value_mut()) {
        *fresh.decor_mut() = decor;
    }
}

#[cfg(test)]
mod tests {
    use super::Editor;
    use crate::load::parse;

    const ARQUIVO: &str = "# meu sparky\nautostart = [\"waybar\"]\n\n[general]\n# folga entre janelas\ngaps_inner = 5\nborder = 2\n";

    fn editor() -> Editor {
        match Editor::parse(ARQUIVO) {
            Ok(editor) => editor,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn muda_so_a_chave_pedida_e_guarda_os_comentarios() {
        let mut editor = editor();
        editor.set_number("general", "gaps_inner", 12);
        let contents = editor.contents();
        assert!(contents.contains("# meu sparky\n"));
        assert!(contents.contains("# folga entre janelas\ngaps_inner = 12\n"));
        assert!(contents.contains("border = 2\n"));
    }

    #[test]
    fn cria_a_tabela_que_falta_e_o_resultado_continua_valido() {
        let mut editor = editor();
        editor.set_flag("general", "default_binds", false);
        editor.set_text("apps", "terminal", Some(" kitty "));
        editor.set_number("input", "repeat_rate", 30);
        let Ok(config) = parse(&editor.contents()) else {
            panic!("o arquivo editado deveria ser válido");
        };
        assert!(!config.general.default_binds);
        assert_eq!(config.apps.terminal.as_deref(), Some("kitty"));
        assert_eq!(config.input.repeat_rate, 30);
    }

    #[test]
    fn texto_vazio_apaga_a_chave() {
        let mut editor = editor();
        editor.set_text("input", "layout", Some("br"));
        editor.set_text("input", "layout", Some("  "));
        assert!(!editor.contents().contains("layout"));
    }

    #[test]
    fn a_lista_fica_no_topo_antes_das_tabelas() {
        let mut editor = Editor::default();
        editor.set_number("general", "border", 3);
        editor.set_list("autostart", &["mako".to_owned(), "nm-applet --indicator".to_owned()]);
        let contents = editor.contents();
        let lista = contents.find("autostart");
        let tabela = contents.find("[general]");
        assert!(lista < tabela, "{contents}");
        let Ok(config) = parse(&contents) else { panic!("inválido: {contents}") };
        assert_eq!(config.autostart, ["mako", "nm-applet --indicator"]);
    }

    #[test]
    fn arquivo_quebrado_e_recusado() {
        assert!(Editor::parse("[general\nborder = ").is_err());
    }
}
