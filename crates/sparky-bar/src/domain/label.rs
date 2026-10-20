pub fn menu_label(raw: &str) -> String {
    let mut label = String::with_capacity(raw.len());
    let mut characters = raw.chars().peekable();
    while let Some(character) = characters.next() {
        match (character, characters.peek()) {
            ('_', Some('_')) => {
                label.push('_');
                characters.next();
            }
            ('_', _) => {}
            (other, _) => label.push(other),
        }
    }
    label
}

#[cfg(test)]
mod tests {
    use super::menu_label;

    #[test]
    fn tira_o_sublinhado_de_atalho() {
        assert_eq!(menu_label("_Sair"), "Sair");
        assert_eq!(menu_label("Mostrar _janela"), "Mostrar janela");
    }

    #[test]
    fn dois_sublinhados_viram_um() {
        assert_eq!(menu_label("nome__do__arquivo"), "nome_do_arquivo");
    }

    #[test]
    fn rotulo_sem_sublinhado_nao_muda() {
        assert_eq!(menu_label("Configurações"), "Configurações");
    }
}
