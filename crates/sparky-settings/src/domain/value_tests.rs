use sparky_config::{Config, Editor, parse};

use crate::domain::field::Field;
use crate::domain::value::{Invalid, Step, Value, parsed, read, shown, stepped, store};

#[test]
fn le_cada_tipo_de_campo_da_configuracao() {
    let config = Config::default();
    assert_eq!(read(Field::GapsOuter, &config), Value::Number(10));
    assert_eq!(read(Field::DefaultBinds, &config), Value::Flag(true));
    assert_eq!(read(Field::Launcher, &config), Value::Text("sparky-launcher".to_owned()));
    assert_eq!(read(Field::Layout, &config), Value::Text(String::new()));
}

#[test]
fn as_setas_andam_no_passo_e_param_nos_limites() {
    assert_eq!(stepped(Field::FloatWidth, &Value::Number(875), Step::More), Some(Value::Number(900)));
    assert_eq!(stepped(Field::Border, &Value::Number(0), Step::Less), Some(Value::Number(0)));
    assert_eq!(stepped(Field::DefaultBinds, &Value::Flag(true), Step::Less), Some(Value::Flag(false)));
    assert_eq!(stepped(Field::Terminal, &Value::Text("kitty".to_owned()), Step::More), None);
}

#[test]
fn o_texto_digitado_e_validado_pelo_tipo_do_campo() {
    assert_eq!(parsed(Field::GapsInner, " 12 "), Ok(Value::Number(12)));
    assert_eq!(parsed(Field::GapsInner, "doze"), Err(Invalid::NotNumber));
    assert!(matches!(parsed(Field::Border, "50"), Err(Invalid::OutOfRange(_))));
    assert_eq!(parsed(Field::Launcher, "  "), Err(Invalid::Blank));
    assert_eq!(parsed(Field::Terminal, ""), Ok(Value::Text(String::new())));
    assert_eq!(parsed(Field::DefaultBinds, "sim"), Err(Invalid::Toggle));
}

#[test]
fn gravar_no_editor_produz_um_sparky_toml_valido() {
    let mut editor = Editor::default();
    store(Field::RepeatRate, &Value::Number(30), &mut editor);
    store(Field::Terminal, &Value::Text("foot".to_owned()), &mut editor);
    store(Field::DefaultBinds, &Value::Flag(false), &mut editor);
    let Ok(config) = parse(&editor.contents()) else { panic!("inválido: {}", editor.contents()) };
    assert_eq!(config.input.repeat_rate, 30);
    assert_eq!(config.apps.terminal.as_deref(), Some("foot"));
    assert!(!config.general.default_binds);
}

#[test]
fn os_valores_aparecem_em_portugues() {
    assert_eq!(shown(&Value::Flag(true)), "ligado");
    assert_eq!(shown(&Value::Text(String::new())), "automático");
    assert_eq!(shown(&Value::Number(5)), "5");
}
