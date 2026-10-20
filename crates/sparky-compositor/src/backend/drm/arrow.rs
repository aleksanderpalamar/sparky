pub const WIDTH: usize = 12;
pub const HEIGHT: usize = 19;

const SHAPE: [&str; HEIGHT] = [
    "X           ",
    "XX          ",
    "X.X         ",
    "X..X        ",
    "X...X       ",
    "X....X      ",
    "X.....X     ",
    "X......X    ",
    "X.......X   ",
    "X........X  ",
    "X.........X ",
    "X......XXXXX",
    "X...X..X    ",
    "X..XX..X    ",
    "X.X  X..X   ",
    "XX   X..X   ",
    "X     X..X  ",
    "      X..X  ",
    "       XX   ",
];

pub fn pixels() -> Vec<u8> {
    SHAPE
        .iter()
        .flat_map(|row| row.chars())
        .flat_map(|cell| match cell {
            'X' => [0, 0, 0, 255],
            '.' => [255, 255, 255, 255],
            _ => [0, 0, 0, 0],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{HEIGHT, SHAPE, WIDTH, pixels};

    #[test]
    fn todas_as_linhas_tem_a_mesma_largura() {
        assert!(SHAPE.iter().all(|row| row.chars().count() == WIDTH));
    }

    #[test]
    fn quatro_bytes_por_pixel() {
        assert_eq!(pixels().len(), WIDTH * HEIGHT * 4);
    }

    #[test]
    fn a_ponta_e_opaca_e_o_fundo_e_transparente() {
        let pixels = pixels();
        assert_eq!(&pixels[0..4], &[0, 0, 0, 255]);
        assert_eq!(pixels[(WIDTH - 1) * 4 + 3], 0);
    }
}
