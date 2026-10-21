use crate::domain::item::NewItem;
use crate::domain::secret::Secret;
use crate::domain::vault::{Existing, Vault};
use crate::seal::{Kdf, SealError, Sealing, header, open};

const RAPIDO: Kdf = Kdf { memory: 64, passes: 1, lanes: 1 };

fn cofre() -> Vault {
    let mut cofre = Vault::new(3);
    let atributos = [("service".to_owned(), "github.com".to_owned())].into_iter().collect();
    let novo = NewItem { label: "Token do GitHub".to_owned(), attributes: atributos, secret: Secret::new(b"ghp_segredo".to_vec(), "text/plain") };
    cofre.store(novo, Existing::Keep, 4);
    cofre
}

fn selado(senha: &str) -> String {
    let Ok(lacre) = Sealing::create(senha.as_bytes(), RAPIDO) else { panic!("sem lacre") };
    let Ok(arquivo) = lacre.seal(&cofre()) else { panic!("sem selo") };
    arquivo
}

#[test]
fn abre_com_a_senha_certa_e_devolve_o_mesmo_cofre() {
    let aberto = open(&selado("correta"), b"correta").map(|(cofre, _)| cofre);
    assert_eq!(aberto, Ok(cofre()));
}

#[test]
fn senha_errada_nao_abre() {
    assert_eq!(open(&selado("correta"), b"errada").err(), Some(SealError::WrongPassword));
}

#[test]
fn o_arquivo_nao_guarda_nada_em_claro() {
    let arquivo = selado("correta");
    for trecho in ["ghp_segredo", "GitHub", "github.com", "service", "correta"] {
        assert!(!arquivo.contains(trecho), "{trecho} apareceu no arquivo");
    }
}

#[test]
fn o_indice_le_sem_senha() {
    let indice = header(&selado("correta")).map(|cabecalho| cabecalho.index.len());
    assert_eq!(indice, Ok(1));
}

#[test]
fn mexer_no_indice_ou_no_conteudo_impede_a_abertura() {
    let arquivo = selado("correta");
    let Ok(cabecalho) = header(&arquivo) else { panic!("sem cabeçalho") };
    let impressao = &cabecalho.index[0].attributes[0];
    let indice_trocado = arquivo.replace(impressao.as_str(), &"0".repeat(64));
    assert_eq!(open(&indice_trocado, b"correta").err(), Some(SealError::WrongPassword));
    let Ok(mut valor) = serde_json::from_str::<serde_json::Value>(&arquivo) else { panic!("json") };
    valor["sealed"] = serde_json::Value::String("AAAA".repeat(20));
    assert_eq!(open(&valor.to_string(), b"correta").err(), Some(SealError::WrongPassword));
}

#[test]
fn cada_gravacao_usa_um_nonce_novo() {
    let Ok(lacre) = Sealing::create(b"x", RAPIDO) else { panic!("sem lacre") };
    assert_ne!(lacre.seal(&cofre()).ok(), lacre.seal(&cofre()).ok());
}

#[test]
fn o_lacre_aberto_regrava_sem_pedir_a_senha_de_novo() {
    let Ok((_, lacre)) = open(&selado("correta"), b"correta") else { panic!("não abriu") };
    let Ok(regravado) = lacre.seal(&Vault::new(9)) else { panic!("sem selo") };
    assert_eq!(open(&regravado, b"correta").map(|(cofre, _)| cofre), Ok(Vault::new(9)));
}

#[test]
fn arquivo_estranho_e_formato_desconhecido_sao_recusados() {
    assert_eq!(header("não é json").err(), Some(SealError::Corrupt));
    let Ok(mut valor) = serde_json::from_str::<serde_json::Value>(&selado("x")) else { panic!("json") };
    valor["header"]["format"] = serde_json::Value::from(99);
    assert_eq!(header(&valor.to_string()).err(), Some(SealError::Unsupported));
    valor["header"]["format"] = serde_json::Value::from(1);
    valor["header"]["kdf"]["memory"] = serde_json::Value::from(u32::MAX);
    assert_eq!(open(&valor.to_string(), b"x").err(), Some(SealError::Corrupt));
}

#[test]
fn o_debug_do_lacre_nao_mostra_a_chave() {
    let Ok(lacre) = Sealing::create(b"x", RAPIDO) else { panic!("sem lacre") };
    assert_eq!(format!("{lacre:?}"), "Sealing { .. }");
}
