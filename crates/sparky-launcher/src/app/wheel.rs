use iced::mouse::ScrollDelta;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Wheel {
    pending: f32,
}

impl Wheel {
    pub fn turn(&mut self, delta: ScrollDelta, row_height: f32) -> isize {
        let rows = match delta {
            ScrollDelta::Lines { y, .. } => -y,
            ScrollDelta::Pixels { y, .. } => -y / row_height.max(1.0),
        };
        if rows.signum() != self.pending.signum() {
            self.pending = 0.0;
        }
        self.pending += rows;
        let whole = self.pending.trunc();
        self.pending -= whole;
        whole as isize
    }
}

#[cfg(test)]
mod tests {
    use super::Wheel;
    use iced::mouse::ScrollDelta;

    fn pixels(y: f32) -> ScrollDelta {
        ScrollDelta::Pixels { x: 0.0, y }
    }

    #[test]
    fn cada_degrau_da_roda_desce_ou_sobe_uma_linha() {
        let mut roda = Wheel::default();
        assert_eq!(roda.turn(ScrollDelta::Lines { x: 0.0, y: -1.0 }, 116.0), 1);
        assert_eq!(roda.turn(ScrollDelta::Lines { x: 0.0, y: 1.0 }, 116.0), -1);
    }

    #[test]
    fn pixels_pequenos_se_acumulam_ate_dar_uma_linha() {
        let mut roda = Wheel::default();
        let linhas: isize = (0..8).map(|_| roda.turn(pixels(-15.0), 116.0)).sum();
        assert_eq!(linhas, 1);
        assert_eq!(roda.turn(pixels(-15.0), 116.0), 0);
    }

    #[test]
    fn mudar_de_sentido_descarta_o_que_sobrou() {
        let mut roda = Wheel::default();
        roda.turn(pixels(-100.0), 116.0);
        assert_eq!(roda.turn(pixels(30.0), 116.0), 0);
        assert_eq!(roda.turn(pixels(90.0), 116.0), -1);
    }
}
