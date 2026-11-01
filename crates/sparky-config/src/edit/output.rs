use toml_edit::{ArrayOfTables, Item, Table, value};

use super::{Editor, put, replace};
use crate::output::OutputRule;

impl Editor {
    pub fn set_output(&mut self, rule: &OutputRule) {
        let Some(table) = self.output_table(&rule.name) else {
            return;
        };
        put(table, "name", value(rule.name.as_str()));
        put(table, "scale", value(rule.scale.factor()));
        let rotation =
            (!rule.rotation.is_normal()).then(|| value(i64::from(rule.rotation.degrees())));
        let enabled = (!rule.enabled).then(|| value(false));
        replace(table, "mode", rule.mode.map(|mode| value(mode.to_string())));
        replace(table, "rotation", rotation);
        replace(table, "vrr", rule.vrr.map(value));
        replace(table, "enabled", enabled);
    }

    fn output_table(&mut self, name: &str) -> Option<&mut Table> {
        let outputs = self.outputs()?;
        let found = outputs
            .iter()
            .enumerate()
            .filter(|(_, table)| table.get("name").and_then(Item::as_str) == Some(name))
            .map(|(index, _)| index)
            .last();
        let index = match found {
            Some(index) => index,
            None => {
                outputs.push(Table::new());
                outputs.len() - 1
            }
        };
        outputs.get_mut(index)
    }

    fn outputs(&mut self) -> Option<&mut ArrayOfTables> {
        let item = self
            .document
            .entry("output")
            .or_insert(Item::ArrayOfTables(ArrayOfTables::new()));
        if let Some(tables) = inline_tables(item) {
            *item = Item::ArrayOfTables(tables);
        }
        item.as_array_of_tables_mut()
    }
}

fn inline_tables(item: &Item) -> Option<ArrayOfTables> {
    item.as_array()?
        .iter()
        .map(|entry| {
            entry
                .as_inline_table()
                .cloned()
                .map(|table| table.into_table())
        })
        .collect()
}
