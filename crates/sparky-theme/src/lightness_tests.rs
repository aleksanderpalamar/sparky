use iced_core::Color;

use crate::lightness::{apart, lightness};

fn hex(value: u32) -> Color {
    Color::from_rgb8((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

fn luminance(color: Color) -> f32 {
    let linear = |channel: f32| match channel <= 0.04045 {
        true => channel / 12.92,
        false => ((channel + 0.055) / 1.055).powf(2.4),
    };
    0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
}

fn contrast(one: Color, other: Color) -> f32 {
    let (a, b) = (luminance(one), luminance(other));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[test]
fn em_fundo_escuro_o_texto_clareia_pela_distancia_pedida() {
    let texto = hex(0x6c7086);
    let novo = apart(texto, hex(0x1e1e2e), 0.24);
    assert!((lightness(novo) - lightness(texto) - 0.24).abs() < 0.01);
}

#[test]
fn em_fundo_claro_o_texto_escurece() {
    let texto = hex(0x8c8fa1);
    let novo = apart(texto, hex(0xeff1f5), 0.24);
    assert!(lightness(novo) < lightness(texto) - 0.2);
}

#[test]
fn sem_distancia_extra_o_texto_fica_igual() {
    let texto = hex(0x6c7086);
    let novo = apart(texto, hex(0x1e1e2e), 0.0);
    assert!((novo.r - texto.r).abs() < 0.005);
    assert!((novo.g - texto.g).abs() < 0.005);
    assert!((novo.b - texto.b).abs() < 0.005);
}

#[test]
fn o_branco_continua_branco_e_a_transparencia_e_mantida() {
    let branco = Color {
        a: 0.5,
        ..Color::WHITE
    };
    let novo = apart(branco, hex(0x1e1e2e), 0.24);
    assert!(novo.r > 0.99 && novo.g > 0.99 && novo.b > 0.99);
    assert_eq!(novo.a, 0.5);
}

#[test]
fn titulo_de_secao_volta_a_ser_legivel_sobre_papel_de_parede_claro() {
    let fundo_com_vidro = hex(0x5b4d70);
    let titulo = hex(0x6c7086);
    assert!(contrast(titulo, fundo_com_vidro) < 2.0);
    let novo = apart(titulo, hex(0x1e1e2e), 0.24);
    assert!(contrast(novo, fundo_com_vidro) >= 3.0);
}
