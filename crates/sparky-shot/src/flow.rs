use std::path::Path;

use sparky_ipc::CaptureArea;

use crate::ports::{Capturer, Clipboard, Notifier};

pub struct Ports<'a> {
    pub capturer: &'a dyn Capturer,
    pub clipboard: &'a dyn Clipboard,
    pub notifier: &'a dyn Notifier,
}

pub fn take(area: &CaptureArea, path: &Path, ports: &Ports<'_>) -> Result<(), String> {
    if let Err(error) = ports.capturer.capture(area, path) {
        ports.notifier.failed(&error);
        return Err(error);
    }
    let copied = ports.clipboard.copy_png(path).is_ok();
    ports.notifier.saved(path, copied);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Ports, take};
    use crate::ports::{Capturer, Clipboard, Notifier};
    use sparky_ipc::CaptureArea;
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};

    #[derive(Default)]
    struct Fakes {
        capture_fails: bool,
        clipboard_fails: bool,
        captured: RefCell<Vec<(CaptureArea, PathBuf)>>,
        notices: RefCell<Vec<String>>,
    }

    impl Capturer for Fakes {
        fn capture(&self, area: &CaptureArea, path: &Path) -> Result<(), String> {
            self.captured.borrow_mut().push((area.clone(), path.to_path_buf()));
            match self.capture_fails {
                true => Err("monitor não encontrado".to_owned()),
                false => Ok(()),
            }
        }
    }

    impl Clipboard for Fakes {
        fn copy_png(&self, _path: &Path) -> Result<(), String> {
            match self.clipboard_fails {
                true => Err("sem wl-copy".to_owned()),
                false => Ok(()),
            }
        }
    }

    impl Notifier for Fakes {
        fn saved(&self, path: &Path, copied: bool) {
            self.notices.borrow_mut().push(format!("salvo {} {copied}", path.display()));
        }

        fn failed(&self, message: &str) {
            self.notices.borrow_mut().push(format!("falhou {message}"));
        }
    }

    fn rodar(fakes: &Fakes) -> Result<(), String> {
        let ports = Ports { capturer: fakes, clipboard: fakes, notifier: fakes };
        take(&CaptureArea::Window, Path::new("/i/captura.png"), &ports)
    }

    #[test]
    fn captura_copia_e_avisa_onde_salvou() {
        let fakes = Fakes::default();
        assert_eq!(rodar(&fakes), Ok(()));
        assert_eq!(fakes.captured.borrow()[0], (CaptureArea::Window, PathBuf::from("/i/captura.png")));
        assert_eq!(*fakes.notices.borrow(), vec!["salvo /i/captura.png true"]);
    }

    #[test]
    fn sem_area_de_transferencia_ainda_salva_e_avisa() {
        let fakes = Fakes { clipboard_fails: true, ..Fakes::default() };
        assert_eq!(rodar(&fakes), Ok(()));
        assert_eq!(*fakes.notices.borrow(), vec!["salvo /i/captura.png false"]);
    }

    #[test]
    fn falha_na_captura_avisa_e_nao_copia() {
        let fakes = Fakes { capture_fails: true, ..Fakes::default() };
        assert_eq!(rodar(&fakes), Err("monitor não encontrado".to_owned()));
        assert_eq!(*fakes.notices.borrow(), vec!["falhou monitor não encontrado"]);
    }
}
