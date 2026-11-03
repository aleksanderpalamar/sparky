mod codes;
mod conversation;
mod library;
mod transaction;

use std::ffi::CString;
use std::path::PathBuf;
use std::ptr;

use zeroize::Zeroizing;

use crate::adapters::pam::codes::SUCCESS;
use crate::adapters::pam::conversation::{Answers, Conversation};
use crate::adapters::pam::library::{Handle, api};
use crate::adapters::pam::transaction::PamTransaction;
use crate::ports::pam::{Pam, Transaction};

#[derive(Debug, Clone, Default)]
pub struct LinuxPam {
    pub confdir: Option<PathBuf>,
}

impl Pam for LinuxPam {
    fn begin(
        &self,
        service: &str,
        user: &str,
        password: Option<Zeroizing<String>>,
    ) -> Result<Box<dyn Transaction>, String> {
        let api = api()?;
        let service = text(service)?;
        let username = text(user)?;
        let password = password
            .map(|password| text(&password).map(Zeroizing::new))
            .transpose()?;
        let answers = Box::new(Answers { username, password });
        let conversation = Box::new(Conversation::new(&answers));
        let mut handle: Handle = ptr::null_mut();
        let code = match (&self.confdir, api.start_in) {
            (None, _) => unsafe {
                (api.start)(
                    service.as_ptr(),
                    answers.username.as_ptr(),
                    &*conversation,
                    &mut handle,
                )
            },
            (Some(dir), Some(start_in)) => {
                let dir = text(&dir.to_string_lossy())?;
                unsafe {
                    start_in(
                        service.as_ptr(),
                        answers.username.as_ptr(),
                        &*conversation,
                        dir.as_ptr(),
                        &mut handle,
                    )
                }
            }
            (Some(_), None) => {
                return Err("esta libpam não aceita outra pasta de configuração".to_owned());
            }
        };
        if code != SUCCESS || handle.is_null() {
            return Err(format!(
                "o PAM não abriu o serviço {}: código {code}",
                service.to_string_lossy()
            ));
        }
        Ok(Box::new(PamTransaction {
            api,
            handle,
            status: SUCCESS,
            established: false,
            opened: false,
            _answers: answers,
            _conversation: conversation,
        }))
    }
}

pub fn text(value: &str) -> Result<CString, String> {
    CString::new(value).map_err(|_| "texto com caractere nulo".to_owned())
}
