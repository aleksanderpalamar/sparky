use std::ffi::{CStr, c_int};

use sparky_login_core::protocol::Refusal;

use crate::adapters::pam::codes::{DELETE_CRED, ESTABLISH_CRED, SUCCESS, TTY, refusal};
use crate::adapters::pam::conversation::{Answers, Conversation};
use crate::adapters::pam::library::{Api, Handle};
use crate::adapters::pam::text;
use crate::ports::pam::Transaction;

pub struct PamTransaction {
    pub api: &'static Api,
    pub handle: Handle,
    pub status: c_int,
    pub established: bool,
    pub opened: bool,
    pub _answers: Box<Answers>,
    pub _conversation: Box<Conversation>,
}

impl PamTransaction {
    fn step(&mut self, code: c_int) -> Result<(), String> {
        self.status = code;
        match code {
            SUCCESS => Ok(()),
            _ => Err(self.describe(code)),
        }
    }

    fn describe(&self, code: c_int) -> String {
        let text = unsafe { (self.api.describe)(self.handle, code) };
        match text.is_null() {
            true => format!("PAM: código {code}"),
            false => format!("PAM: {}", unsafe { CStr::from_ptr(text) }.to_string_lossy()),
        }
    }

    fn check(&mut self, code: c_int) -> Result<(), Refusal> {
        self.status = code;
        match code {
            SUCCESS => Ok(()),
            _ => Err(refusal(code, || self.describe(code))),
        }
    }
}

impl Transaction for PamTransaction {
    fn authenticate(&mut self) -> Result<(), Refusal> {
        let code = unsafe { (self.api.authenticate)(self.handle, 0) };
        self.check(code)?;
        let code = unsafe { (self.api.account)(self.handle, 0) };
        self.check(code)
    }

    fn set_tty(&mut self, tty: &str) -> Result<(), String> {
        let tty = text(tty)?;
        let code = unsafe { (self.api.set_item)(self.handle, TTY, tty.as_ptr().cast()) };
        self.step(code)
    }

    fn put(&mut self, name: &str, value: &str) -> Result<(), String> {
        let pair = text(&format!("{name}={value}"))?;
        let code = unsafe { (self.api.put_env)(self.handle, pair.as_ptr()) };
        self.step(code)
    }

    fn establish(&mut self) -> Result<(), String> {
        let code = unsafe { (self.api.set_credentials)(self.handle, ESTABLISH_CRED) };
        self.step(code)?;
        self.established = true;
        Ok(())
    }

    fn open(&mut self) -> Result<(), String> {
        let code = unsafe { (self.api.open_session)(self.handle, 0) };
        self.step(code)?;
        self.opened = true;
        Ok(())
    }

    fn environment(&self) -> Vec<(String, String)> {
        let list = unsafe { (self.api.env_list)(self.handle) };
        if list.is_null() {
            return Vec::new();
        }
        let mut pairs = Vec::new();
        let mut index = 0;
        loop {
            let entry = unsafe { *list.add(index) };
            if entry.is_null() {
                break;
            }
            let line = unsafe { CStr::from_ptr(entry) }
                .to_string_lossy()
                .into_owned();
            if let Some((name, value)) = line.split_once('=') {
                pairs.push((name.to_owned(), value.to_owned()));
            }
            unsafe { libc::free(entry.cast()) };
            index += 1;
        }
        unsafe { libc::free(list.cast()) };
        pairs
    }

    fn close(&mut self) {
        if self.opened {
            let code = unsafe { (self.api.close_session)(self.handle, 0) };
            self.opened = false;
            if let Err(error) = self.step(code) {
                eprintln!("sparky-login: fechar a sessão: {error}");
            }
        }
        if self.established {
            let code = unsafe { (self.api.set_credentials)(self.handle, DELETE_CRED) };
            self.established = false;
            if let Err(error) = self.step(code) {
                eprintln!("sparky-login: apagar as credenciais: {error}");
            }
        }
    }
}

impl Drop for PamTransaction {
    fn drop(&mut self) {
        self.close();
        unsafe { (self.api.end)(self.handle, self.status) };
    }
}
