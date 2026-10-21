use crate::domain::index::{fingerprint, index_of, search};
use crate::domain::item::{Attributes, NewItem};
use crate::domain::secret::Secret;
use crate::domain::vault::{Existing, Vault};

fn atributos(pares: &[(&str, &str)]) -> Attributes {
    pares.iter().map(|(nome, valor)| (nome.to_string(), valor.to_string())).collect()
}

fn cofre() -> Vault {
    let mut cofre = Vault::new(0);
    for (rotulo, usuario) in [("a", "ana"), ("b", "bia")] {
        let novo = NewItem { label: rotulo.to_owned(), attributes: atributos(&[("service", "git"), ("user", usuario)]), secret: Secret::new(vec![1], "text/plain") };
        cofre.store(novo, Existing::Keep, 1);
    }
    cofre
}

#[test]
fn o_indice_nao_guarda_os_atributos_em_claro() {
    let indice = index_of(&cofre());
    let texto = format!("{indice:?}");
    assert!(!texto.contains("git"));
    assert!(!texto.contains("ana"));
    assert_eq!(indice.len(), 2);
    assert_eq!(fingerprint("service", "git").len(), 64);
}

#[test]
fn o_resumo_separa_nome_e_valor() {
    assert_ne!(fingerprint("ab", "c"), fingerprint("a", "bc"));
}

#[test]
fn com_o_cofre_trancado_a_busca_usa_o_indice() {
    let indice = index_of(&cofre());
    assert_eq!(search(&indice, &atributos(&[("service", "git")])), vec![1, 2]);
    assert_eq!(search(&indice, &atributos(&[("user", "bia"), ("service", "git")])), vec![2]);
    assert!(search(&indice, &atributos(&[("user", "caio")])).is_empty());
}
