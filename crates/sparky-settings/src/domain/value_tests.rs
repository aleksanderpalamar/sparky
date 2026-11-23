use sparky_config::{Config, Editor, parse};

use crate::domain::field::Field;
use crate::domain::value::{Invalid, Step, Value, parsed, read, shown, stepped, store};

#[test]
fn le_cada_tipo_de_campo_da_configuracao() {
    let config = Config::default();
    assert_eq!(read(Field::GapsOuter, &config), Value::Number(10));
    assert_eq!(read(Field::DefaultBinds, &config), Value::Flag(true));
    assert_eq!(
        read(Field::Launcher, &config),
        Value::Text("sparky-launcher".to_owned())
    );
    assert_eq!(read(Field::Terminal, &config), Value::Text(String::new()));
}

#[test]
fn as_setas_andam_no_passo_e_param_nos_limites() {
    assert_eq!(
        stepped(Field::FloatWidth, &Value::Number(875), Step::More),
        Some(Value::Number(900))
    );
    assert_eq!(
        stepped(Field::Border, &Value::Number(0), Step::Less),
        Some(Value::Number(0))
    );
    assert_eq!(
        stepped(Field::DefaultBinds, &Value::Flag(true), Step::Less),
        Some(Value::Flag(false))
    );
    assert_eq!(
        stepped(
            Field::Terminal,
            &Value::Text("kitty".to_owned()),
            Step::More
        ),
        None
    );
}

#[test]
fn o_texto_digitado_e_validado_pelo_tipo_do_campo() {
    assert_eq!(parsed(Field::GapsInner, " 12 "), Ok(Value::Number(12)));
    assert_eq!(parsed(Field::GapsInner, "doze"), Err(Invalid::NotNumber));
    assert!(matches!(
        parsed(Field::Border, "50"),
        Err(Invalid::OutOfRange(_))
    ));
    assert_eq!(parsed(Field::Launcher, "  "), Err(Invalid::Blank));
    assert_eq!(parsed(Field::Terminal, ""), Ok(Value::Text(String::new())));
    assert_eq!(parsed(Field::DefaultBinds, "sim"), Err(Invalid::Toggle));
}

#[test]
fn gravar_no_editor_produz_um_sparky_toml_valido() {
    let mut editor = Editor::default();
    store(Field::RepeatRate, &Value::Number(30), &mut editor);
    store(
        Field::Terminal,
        &Value::Text("foot".to_owned()),
        &mut editor,
    );
    store(Field::DefaultBinds, &Value::Flag(false), &mut editor);
    let Ok(config) = parse(&editor.contents()) else {
        panic!("inválido: {}", editor.contents())
    };
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

#[test]
fn le_e_grava_as_opcoes_do_dock() {
    let config = Config::default();
    assert_eq!(read(Field::DockEnabled, &config), Value::Flag(true));
    assert_eq!(read(Field::DockAutohide, &config), Value::Flag(false));
    assert_eq!(read(Field::DockSize, &config), Value::Number(64));
    let mut editor = Editor::default();
    store(Field::DockAutohide, &Value::Flag(true), &mut editor);
    store(Field::DockSize, &Value::Number(48), &mut editor);
    let Ok(saved) = parse(&editor.contents()) else {
        panic!("o arquivo gravado deveria ser válido");
    };
    assert_eq!(read(Field::DockAutohide, &saved), Value::Flag(true));
    assert_eq!(read(Field::DockSize, &saved), Value::Number(48));
    assert_eq!(read(Field::DefaultBinds, &saved), Value::Flag(true));
}

#[test]
fn o_tamanho_do_dock_anda_de_quatro_em_quatro_dentro_da_faixa() {
    assert_eq!(
        stepped(Field::DockSize, &Value::Number(64), Step::More),
        Some(Value::Number(68))
    );
    assert_eq!(
        stepped(Field::DockSize, &Value::Number(96), Step::More),
        Some(Value::Number(96))
    );
}

#[test]
fn le_e_grava_os_cantos_arredondados_do_dock() {
    let config = Config::default();
    assert_eq!(read(Field::DockRounded, &config), Value::Flag(false));
    assert_eq!(read(Field::DockRadius, &config), Value::Number(4));
    let mut editor = Editor::default();
    store(Field::DockRounded, &Value::Flag(true), &mut editor);
    store(Field::DockRadius, &Value::Number(3), &mut editor);
    let Ok(saved) = parse(&editor.contents()) else {
        panic!("o arquivo gravado deveria ser válido");
    };
    assert_eq!(read(Field::DockRounded, &saved), Value::Flag(true));
    assert_eq!(read(Field::DockRadius, &saved), Value::Number(3));
    assert_eq!(
        stepped(Field::DockRadius, &Value::Number(16), Step::More),
        Some(Value::Number(16))
    );
}
