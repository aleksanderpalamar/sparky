use smithay::utils::Transform;
use sparky_config::Rotation;

pub fn transform(rotation: Rotation) -> Transform {
    match rotation {
        Rotation::Normal => Transform::Normal,
        Rotation::Right => Transform::_90,
        Rotation::Inverted => Transform::_180,
        Rotation::Left => Transform::_270,
    }
}

pub fn rotation(transform: Transform) -> Rotation {
    match transform {
        Transform::_90 | Transform::Flipped90 => Rotation::Right,
        Transform::_180 | Transform::Flipped180 => Rotation::Inverted,
        Transform::_270 | Transform::Flipped270 => Rotation::Left,
        Transform::Normal | Transform::Flipped => Rotation::Normal,
    }
}

#[cfg(test)]
mod tests {
    use super::{rotation, transform};
    use sparky_config::Rotation;

    #[test]
    fn os_quatro_giros_viram_transformacoes_e_voltam() {
        for giro in Rotation::ALL {
            assert_eq!(rotation(transform(giro)), giro);
        }
    }
}
