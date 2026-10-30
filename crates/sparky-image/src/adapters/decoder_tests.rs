use std::fs;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use image::{ImageBuffer, ImageFormat, Rgb};

use super::decoder::ImageFile;
use crate::ports::{DecodeError, Decoder};

fn jpeg_girado(largura: u32, altura: u32) -> Vec<u8> {
    let mut jpeg = Vec::new();
    ImageBuffer::from_pixel(largura, altura, Rgb([200u8, 30, 30]))
        .write_to(&mut std::io::Cursor::new(&mut jpeg), ImageFormat::Jpeg)
        .unwrap();
    let exif: [u8; 36] = [
        0xFF, 0xE1, 0x00, 0x22, b'E', b'x', b'i', b'f', 0, 0, b'I', b'I', 0x2A, 0x00, 0x08, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x12, 0x01, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    [&jpeg[..2], &exif[..], &jpeg[2..]].concat()
}

fn bmp_gigante() -> Vec<u8> {
    let mut bmp = b"BM".to_vec();
    for valor in [54u32, 0, 54, 40, 20_000, 20_000] {
        bmp.extend(valor.to_le_bytes());
    }
    bmp.extend(1u16.to_le_bytes());
    bmp.extend(24u16.to_le_bytes());
    bmp.extend([0u8; 24]);
    bmp
}

#[test]
fn decodifica_png_em_rgba() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tela.png");
    ImageBuffer::from_pixel(6, 3, Rgb([1u8, 2, 3]))
        .save(&path)
        .unwrap();
    let picture = ImageFile.decode(&path).unwrap();
    assert_eq!((picture.width, picture.height), (6, 3));
    assert_eq!(picture.rgba.len(), 6 * 3 * 4);
    assert_eq!(&picture.rgba[..4], &[1, 2, 3, 255]);
}

#[test]
fn foto_com_orientacao_exif_aparece_em_pe() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("celular.jpg");
    fs::write(&path, jpeg_girado(8, 4)).unwrap();
    let picture = ImageFile.decode(&path).unwrap();
    assert_eq!((picture.width, picture.height), (4, 8));
}

#[test]
fn arquivo_corrompido_ou_ausente_vira_erro_explicito() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("quebrada.png");
    fs::write(&path, b"\x89PNG\r\n\x1a\nquebrado").unwrap();
    assert_eq!(ImageFile.decode(&path), Err(DecodeError::Corrupted));
    let texto = dir.path().join("notas.foto");
    fs::write(&texto, b"so texto").unwrap();
    assert_eq!(ImageFile.decode(&texto), Err(DecodeError::Unsupported));
    let ausente = ImageFile.decode(&dir.path().join("sumiu.png"));
    assert!(matches!(ausente, Err(DecodeError::Unreadable(_))));
}

#[test]
fn imagem_gigante_e_recusada_sem_alocar_os_pixels() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("enorme.bmp");
    fs::write(&path, bmp_gigante()).unwrap();
    assert_eq!(ImageFile.decode(&path), Err(DecodeError::TooLarge));
}

#[test]
fn fifo_com_extensao_de_imagem_nao_trava() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cano.png");
    assert!(
        Command::new("mkfifo")
            .arg(&path)
            .status()
            .unwrap()
            .success()
    );
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let _ = sender.send(ImageFile.decode(&path));
    });
    let resultado = receiver.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(resultado, Err(DecodeError::NotAFile));
}

#[test]
fn erros_tem_mensagens_em_portugues() {
    assert_eq!(
        DecodeError::TooLarge.to_string(),
        "Imagem grande demais (máximo de 16384 pixels por lado)"
    );
    assert_eq!(
        DecodeError::Unsupported.to_string(),
        "Formato de imagem não suportado"
    );
}
