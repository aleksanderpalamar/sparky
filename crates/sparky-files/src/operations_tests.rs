use std::cell::RefCell;
use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::domain::clip::Clip;
use crate::domain::entry::Entry;
use crate::operations::{Failure, create, paste, rename, trash};
use crate::ports::Disk;

#[derive(Default)]
struct FakeDisk {
    existing: RefCell<BTreeSet<PathBuf>>,
    calls: RefCell<Vec<String>>,
    broken: bool,
}

impl FakeDisk {
    fn with(paths: &[&str]) -> Self {
        let disk = Self::default();
        disk.existing.borrow_mut().extend(paths.iter().map(PathBuf::from));
        disk
    }

    fn record(&self, call: String, created: Option<&Path>) -> io::Result<()> {
        if self.broken {
            return Err(io::Error::other("disco cheio"));
        }
        self.calls.borrow_mut().push(call);
        if let Some(path) = created {
            self.existing.borrow_mut().insert(path.to_path_buf());
        }
        Ok(())
    }

    fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl Disk for FakeDisk {
    fn list(&self, _dir: &Path) -> io::Result<Vec<Entry>> {
        Ok(Vec::new())
    }

    fn exists(&self, path: &Path) -> bool {
        self.existing.borrow().contains(path)
    }

    fn stamp(&self, _dir: &Path) -> Option<SystemTime> {
        None
    }

    fn create_dir(&self, path: &Path) -> io::Result<()> {
        self.record(format!("pasta {}", path.display()), Some(path))
    }

    fn create_file(&self, path: &Path) -> io::Result<()> {
        self.record(format!("arquivo {}", path.display()), Some(path))
    }

    fn relocate(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.record(format!("mover {} {}", from.display(), to.display()), Some(to))
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.record(format!("copiar {} {}", from.display(), to.display()), Some(to))
    }

    fn trash(&self, path: &Path) -> io::Result<()> {
        self.record(format!("lixeira {}", path.display()), None)
    }
}

fn copia(paths: &[&str]) -> Clip {
    Clip::Copy(paths.iter().map(PathBuf::from).collect())
}

fn recorte(paths: &[&str]) -> Clip {
    Clip::Cut(paths.iter().map(PathBuf::from).collect())
}

#[test]
fn copiar_para_a_mesma_pasta_ganha_nome_de_copia() {
    let disk = FakeDisk::with(&["/h/foto.png"]);
    let pasted = paste(&copia(&["/h/foto.png"]), Path::new("/h"), &disk);
    assert_eq!(pasted.ok(), Some(vec!["foto (cópia).png".to_owned()]));
    assert_eq!(disk.calls(), ["copiar /h/foto.png /h/foto (cópia).png"]);
}

#[test]
fn recortar_para_a_mesma_pasta_nao_mexe_em_nada() {
    let disk = FakeDisk::with(&["/h/foto.png"]);
    let pasted = paste(&recorte(&["/h/foto.png"]), Path::new("/h"), &disk);
    assert_eq!(pasted.ok(), Some(vec!["foto.png".to_owned()]));
    assert!(disk.calls().is_empty());
}

#[test]
fn recortar_para_outra_pasta_move_cada_item() {
    let disk = FakeDisk::with(&["/h/a.txt", "/h/b.txt"]);
    let pasted = paste(&recorte(&["/h/a.txt", "/h/b.txt"]), Path::new("/h/Docs"), &disk);
    assert_eq!(pasted.ok(), Some(vec!["a.txt".to_owned(), "b.txt".to_owned()]));
    assert_eq!(disk.calls(), ["mover /h/a.txt /h/Docs/a.txt", "mover /h/b.txt /h/Docs/b.txt"]);
}

#[test]
fn colar_uma_pasta_dentro_dela_mesma_e_recusado() {
    let disk = FakeDisk::with(&["/h/Fotos"]);
    let pasted = paste(&copia(&["/h/Fotos"]), Path::new("/h/Fotos/2026"), &disk);
    assert!(matches!(pasted, Err(Failure::IntoItself(name)) if name == "Fotos"));
    assert!(disk.calls().is_empty());
}

#[test]
fn renomear_move_dentro_da_pasta() {
    let disk = FakeDisk::with(&["/h/velho.txt"]);
    assert!(rename(Path::new("/h"), "velho.txt", "novo.txt", &disk).is_ok());
    assert_eq!(disk.calls(), ["mover /h/velho.txt /h/novo.txt"]);
}

#[test]
fn renomear_para_nome_ocupado_ou_invalido_e_recusado() {
    let disk = FakeDisk::with(&["/h/a.txt", "/h/b.txt"]);
    assert!(matches!(rename(Path::new("/h"), "a.txt", "b.txt", &disk), Err(Failure::AlreadyExists(_))));
    assert!(matches!(rename(Path::new("/h"), "a.txt", "x/y", &disk), Err(Failure::InvalidName)));
    assert!(rename(Path::new("/h"), "a.txt", "a.txt", &disk).is_ok());
    assert!(disk.calls().is_empty());
}

#[test]
fn criar_pasta_ou_arquivo_conforme_a_barra() {
    let disk = FakeDisk::default();
    assert_eq!(create(Path::new("/h"), "Projetos/", &disk).ok(), Some("Projetos".to_owned()));
    assert_eq!(create(Path::new("/h"), "notas.md", &disk).ok(), Some("notas.md".to_owned()));
    assert_eq!(disk.calls(), ["pasta /h/Projetos", "arquivo /h/notas.md"]);
    assert!(matches!(create(Path::new("/h"), "notas.md", &disk), Err(Failure::AlreadyExists(_))));
}

#[test]
fn a_falha_do_disco_diz_qual_item_falhou() {
    let disk = FakeDisk {
        broken: true,
        ..FakeDisk::default()
    };
    let Err(failure) = trash(Path::new("/h"), &["a.txt".to_owned()], &disk) else {
        panic!("a lixeira deveria falhar");
    };
    assert_eq!(failure.to_string(), "a.txt: disco cheio");
}

#[test]
fn a_lixeira_recebe_cada_item_escolhido() {
    let disk = FakeDisk::default();
    let names = ["a.txt".to_owned(), "Fotos".to_owned()];
    assert_eq!(trash(Path::new("/h"), &names, &disk).ok(), Some(2));
    assert_eq!(disk.calls(), ["lixeira /h/a.txt", "lixeira /h/Fotos"]);
}
