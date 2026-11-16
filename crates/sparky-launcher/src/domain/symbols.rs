use crate::domain::category::CATEGORIES;
use crate::domain::icon::Icon;
use crate::domain::shelf::Shelf;

pub const ADD: &str = "@add";
const HOME: &str = "@home";
const GROUP: &str = "@group";
const HOME_ICON: &[&str] = &["user-home-symbolic", "go-home-symbolic"];
const GROUP_ICON: &[&str] = &["folder-symbolic"];
const ADD_ICON: &[&str] = &["list-add-symbolic"];

pub fn key(shelf: Shelf) -> String {
    match shelf {
        Shelf::Home => HOME.to_owned(),
        Shelf::Category(category) => format!("@{}", category.label()),
        Shelf::Group(_) => GROUP.to_owned(),
    }
}

pub fn symbols() -> Vec<(String, Icon)> {
    let categories = CATEGORIES.into_iter().map(|category| {
        (
            key(Shelf::Category(category)),
            Icon::Symbolic(category.icon()),
        )
    });
    [
        (HOME.to_owned(), Icon::Symbolic(HOME_ICON)),
        (GROUP.to_owned(), Icon::Symbolic(GROUP_ICON)),
        (ADD.to_owned(), Icon::Symbolic(ADD_ICON)),
    ]
    .into_iter()
    .chain(categories)
    .collect()
}

#[cfg(test)]
mod tests {
    use super::{key, symbols};
    use crate::domain::category::Category;
    use crate::domain::shelf::Shelf;

    #[test]
    fn toda_prateleira_tem_um_icone_simbolico_pedido() {
        let pedidos: Vec<String> = symbols().into_iter().map(|(key, _)| key).collect();
        for shelf in [
            Shelf::Home,
            Shelf::Group(3),
            Shelf::Category(Category::Games),
        ] {
            assert!(pedidos.contains(&key(shelf)));
        }
        assert!(pedidos.iter().all(|key| key.starts_with('@')));
    }
}
