use std::time::Instant;

use iced::window;

use super::drawers::Drawers;
use crate::domain::reveal::{DURATION, Expansion};

#[test]
fn cada_monitor_anima_somente_a_propria_bandeja_e_para_em_repouso() {
    let mut drawers = Drawers::default();
    let first = window::Id::unique();
    let second = window::Id::unique();
    let now = Instant::now();
    assert!(!drawers.animating());
    drawers.hover(first, Expansion::Open, false, now);
    assert!(drawers.animating());
    drawers.advance(now + DURATION);
    assert_eq!(drawers.progress(first), 1.0);
    assert_eq!(drawers.progress(second), 0.0);
    assert!(!drawers.animating());
    drawers.hover(first, Expansion::Closed, false, now + DURATION);
    drawers.advance(now + DURATION * 2);
    assert_eq!(drawers.progress(first), 0.0);
    assert!(!drawers.animating());
}

#[test]
fn um_menu_aberto_mantem_a_gaveta_visivel_ate_fechar() {
    let mut drawers = Drawers::default();
    let window = window::Id::unique();
    let now = Instant::now();
    drawers.hover(window, Expansion::Open, false, now);
    drawers.advance(now + DURATION);
    drawers.hover(window, Expansion::Closed, true, now + DURATION);
    drawers.advance(now + DURATION * 2);
    assert_eq!(drawers.progress(window), 1.0);
    drawers.release(now + DURATION * 2);
    drawers.advance(now + DURATION * 3);
    assert_eq!(drawers.progress(window), 0.0);
}
