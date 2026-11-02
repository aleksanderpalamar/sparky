use super::description::{Chromaticities, NamedPrimaries, Xy};

pub type Matrix = [[f64; 3]; 3];

const COORDINATE_SCALE: f64 = 1_000_000.0;
const BRADFORD: Matrix = [
    [0.8951, 0.2664, -0.1614],
    [-0.7502, 1.7135, 0.0367],
    [0.0389, -0.0685, 1.0296],
];

pub fn to_bt709(primaries: NamedPrimaries) -> Matrix {
    let source = primaries.chromaticities();
    let target = NamedPrimaries::Srgb.chromaticities();
    let adapted = multiply(&adaptation(source.white, target.white), &to_xyz(&source));
    multiply(&invert(&to_xyz(&target)), &adapted)
}

pub fn apply(matrix: &Matrix, rgb: [f64; 3]) -> [f64; 3] {
    matrix.map(|row| {
        row.iter()
            .zip(rgb)
            .map(|(weight, channel)| weight * channel)
            .sum()
    })
}

fn to_xyz(gamut: &Chromaticities) -> Matrix {
    let [red, green, blue] = [gamut.red, gamut.green, gamut.blue].map(tristimulus);
    let corners = transpose(&[red, green, blue]);
    let weights = apply(&invert(&corners), tristimulus(gamut.white));
    corners.map(|row| [0, 1, 2].map(|column| row[column] * weights[column]))
}

fn adaptation(from: Xy, to: Xy) -> Matrix {
    let source = apply(&BRADFORD, tristimulus(from));
    let target = apply(&BRADFORD, tristimulus(to));
    let diagonal = [0, 1, 2].map(|row| {
        [0, 1, 2].map(|column| match row == column {
            true => target[row] / source[row],
            false => 0.0,
        })
    });
    multiply(&invert(&BRADFORD), &multiply(&diagonal, &BRADFORD))
}

fn tristimulus(point: Xy) -> [f64; 3] {
    let x = f64::from(point.x) / COORDINATE_SCALE;
    let y = f64::from(point.y) / COORDINATE_SCALE;
    [x / y, 1.0, (1.0 - x - y) / y]
}

fn multiply(left: &Matrix, right: &Matrix) -> Matrix {
    let columns = transpose(right);
    left.map(|row| columns.map(|column| row.iter().zip(column).map(|(a, b)| a * b).sum()))
}

fn transpose(matrix: &Matrix) -> Matrix {
    [0, 1, 2].map(|row| [0, 1, 2].map(|column| matrix[column][row]))
}

fn invert(m: &Matrix) -> Matrix {
    let cofactor = |row: usize, column: usize| {
        let (r1, r2) = ((row + 1) % 3, (row + 2) % 3);
        let (c1, c2) = ((column + 1) % 3, (column + 2) % 3);
        m[r1][c1] * m[r2][c2] - m[r1][c2] * m[r2][c1]
    };
    let determinant: f64 = (0..3)
        .map(|column| m[0][column] * cofactor(0, column))
        .sum();
    [0, 1, 2].map(|row| [0, 1, 2].map(|column| cofactor(column, row) / determinant))
}

#[cfg(test)]
mod tests {
    use super::{apply, to_bt709};
    use crate::color_management::description::NamedPrimaries;

    fn perto(a: [f64; 3], b: [f64; 3]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4)
    }

    #[test]
    fn srgb_nao_muda() {
        let matrix = to_bt709(NamedPrimaries::Srgb);
        assert!(perto(apply(&matrix, [0.2, 0.5, 0.9]), [0.2, 0.5, 0.9]));
    }

    #[test]
    fn bt2020_vira_bt709_pela_matriz_da_bt2087() {
        let matrix = to_bt709(NamedPrimaries::Bt2020);
        assert!(perto(
            apply(&matrix, [1.0, 0.0, 0.0]),
            [1.660_5, -0.124_6, -0.018_2]
        ));
        assert!(perto(
            apply(&matrix, [0.0, 1.0, 0.0]),
            [-0.587_6, 1.132_9, -0.100_6]
        ));
        assert!(perto(
            apply(&matrix, [0.0, 0.0, 1.0]),
            [-0.072_8, -0.008_3, 1.118_7]
        ));
    }

    #[test]
    fn display_p3_tem_vermelho_fora_do_srgb() {
        let vermelho = apply(&to_bt709(NamedPrimaries::DisplayP3), [1.0, 0.0, 0.0]);
        assert!(
            perto(vermelho, [1.224_9, -0.042_1, -0.019_6]),
            "{vermelho:?}"
        );
    }

    #[test]
    fn branco_de_qualquer_primaria_continua_branco() {
        for primaries in NamedPrimaries::ALL {
            let branco = apply(&to_bt709(primaries), [1.0, 1.0, 1.0]);
            assert!(perto(branco, [1.0, 1.0, 1.0]), "{primaries:?}: {branco:?}");
        }
    }
}
