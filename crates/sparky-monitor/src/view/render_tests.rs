use std::sync::Arc;

use iced::advanced::layout::{Layout, Limits};
use iced::advanced::renderer::{Headless, Style};
use iced::advanced::widget::{Id, Operation, Tree};
use iced::futures::executor::block_on;
use iced::{Pixels, Rectangle, Renderer, Size, mouse};

use crate::app::message::Message;
use crate::app::page::Page;
use crate::app::state::Monitor;
use crate::app::test_support::{JANELA, Sinais, monitor};
use crate::app::update::update;

#[derive(Default)]
struct Textos(Vec<String>);

impl Operation for Textos {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn text(&mut self, _: Option<&Id>, _: Rectangle, text: &str) {
        self.0.push(text.to_owned());
    }
}

fn renderizar(monitor: &Monitor, nome: &str) -> Vec<String> {
    let mut renderer = block_on(<Renderer as Headless>::new(
        sparky_ui::style::FONT,
        Pixels(14.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    let mut view = super::view(monitor);
    let mut tree = Tree::new(&view);
    let node = view
        .as_widget_mut()
        .layout(&mut tree, &renderer, &Limits::new(Size::ZERO, JANELA));
    view.as_widget().draw(
        &tree,
        &mut renderer,
        &monitor.skin.theme(),
        &Style {
            text_color: monitor.skin.foreground,
        },
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &Rectangle::with_size(JANELA),
    );
    let size = Size::new(JANELA.width as u32, JANELA.height as u32);
    let pixels = renderer.screenshot(size, 1.0, monitor.skin.background);
    if let Ok(pasta) = std::env::var("SPARKY_MONITOR_SNAPSHOTS") {
        let mut bytes = format!("P6\n{} {}\n255\n", size.width, size.height).into_bytes();
        bytes.extend(pixels.chunks_exact(4).flat_map(|pixel| pixel[..3].to_vec()));
        std::fs::write(format!("{pasta}/{nome}.ppm"), bytes).unwrap();
    }
    let mut textos = Textos::default();
    view.as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut textos);
    textos.0
}

fn contem(textos: &[String], esperados: &[&str]) {
    for esperado in esperados {
        assert!(
            textos.iter().any(|texto| texto == esperado),
            "faltou {esperado}: {textos:?}"
        );
    }
}

#[test]
fn painel_mostra_tabelas_cartoes_e_resumo() {
    let monitor = monitor(Arc::new(Sinais::default()));
    let textos = renderizar(&monitor, "painel");
    contem(
        &textos,
        &[
            "Painel",
            "Aplicativos",
            "Processos",
            "Firefox",
            "Web Content",
            "CPU",
            "Memória",
            "Disco",
            "Rede",
            "4 processos · 1 aplicativos",
        ],
    );
    assert!(textos.iter().any(|texto| texto.starts_with("CPU ▾")));
}

#[test]
fn todas_as_paginas_renderizam_com_seu_conteudo() {
    let mut monitor = monitor(Arc::new(Sinais::default()));
    let esperado = [
        (Page::Applications, "Aplicativos"),
        (Page::Processes, "zsh"),
        (Page::Cpu, "Uso total"),
        (Page::Memory, "Swap em uso"),
        (Page::Disk, "Todos os discos"),
        (Page::Network, "Todas as interfaces"),
        (Page::Gpu, "Nenhuma GPU encontrada em /sys/class/drm"),
    ];
    for (page, texto) in esperado {
        let _ = update(&mut monitor, Message::Open(page));
        let textos = renderizar(&monitor, page.label());
        contem(&textos, &[texto]);
    }
}
