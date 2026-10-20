use sparky_tiling::{Fullscreen, Layer, Placement, Rect};

pub fn border_width(placement: &Placement, configured: i32) -> i32 {
    match placement.layer {
        Layer::Fullscreen(Fullscreen::Monitor) => 0,
        _ => configured,
    }
}

pub fn content(frame: Rect, width: i32) -> Rect {
    frame.shrink(width, width, width, width)
}

pub fn edges(frame: Rect, width: i32) -> [Rect; 4] {
    let side_height = (frame.height - 2 * width).max(0);
    [
        Rect::new(frame.x, frame.y, frame.width, width),
        Rect::new(frame.x, frame.bottom() - width, frame.width, width),
        Rect::new(frame.x, frame.y + width, width, side_height),
        Rect::new(frame.right() - width, frame.y + width, width, side_height),
    ]
}

#[cfg(test)]
mod tests {
    use super::{border_width, content, edges};
    use sparky_tiling::{Fullscreen, Layer, Placement, Rect, WindowId};

    fn posicao(layer: Layer) -> Placement {
        Placement {
            window: WindowId(1),
            frame: Rect::new(10, 36, 945, 1034),
            layer,
            focused: true,
        }
    }

    #[test]
    fn o_conteudo_fica_dentro_da_borda() {
        assert_eq!(content(Rect::new(10, 36, 945, 1034), 2), Rect::new(12, 38, 941, 1030));
    }

    #[test]
    fn as_quatro_bordas_fecham_a_moldura_sem_sobrepor() {
        let [top, bottom, left, right] = edges(Rect::new(0, 0, 100, 50), 2);
        assert_eq!(top, Rect::new(0, 0, 100, 2));
        assert_eq!(bottom, Rect::new(0, 48, 100, 2));
        assert_eq!(left, Rect::new(0, 2, 2, 46));
        assert_eq!(right, Rect::new(98, 2, 2, 46));
    }

    #[test]
    fn tela_cheia_do_monitor_nao_tem_borda() {
        assert_eq!(border_width(&posicao(Layer::Fullscreen(Fullscreen::Monitor)), 2), 0);
        assert_eq!(border_width(&posicao(Layer::Fullscreen(Fullscreen::Maximized)), 2), 2);
        assert_eq!(border_width(&posicao(Layer::Tiled), 2), 2);
    }
}
