use std::path::Path;

use image::ImageReader;
use image::imageops::FilterType;

pub fn decode(path: &Path, size: (u32, u32)) -> Result<Vec<u8>, String> {
    let reader = ImageReader::open(path).map_err(|error| error.to_string())?;
    let image = reader.with_guessed_format().map_err(|error| error.to_string())?.decode().map_err(|error| error.to_string())?;
    Ok(image.resize_to_fill(size.0, size.1, FilterType::Triangle).into_rgba8().into_raw())
}

#[cfg(test)]
mod tests {
    use image::{Rgba, RgbaImage};

    use super::decode;

    #[test]
    fn a_imagem_cobre_o_monitor_cortando_as_sobras_do_centro() {
        let Ok(pasta) = tempfile::tempdir() else { panic!("sem pasta temporária") };
        let caminho = pasta.path().join("faixas.png");
        let imagem = RgbaImage::from_fn(6, 2, |x, _| match x {
            0 | 5 => Rgba([255, 0, 0, 255]),
            _ => Rgba([0, 0, 255, 255]),
        });
        assert!(imagem.save(&caminho).is_ok());
        let Ok(pixels) = decode(&caminho, (2, 2)) else { panic!("não decodificou") };
        assert_eq!(pixels.len(), 2 * 2 * 4);
        assert!(pixels.chunks(4).all(|pixel| pixel == [0, 0, 255, 255]));
    }

    #[test]
    fn arquivo_ausente_vira_erro() {
        assert!(decode(std::path::Path::new("/nao/existe.webp"), (2, 2)).is_err());
    }
}
