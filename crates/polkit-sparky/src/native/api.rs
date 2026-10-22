use std::ffi::{CStr, c_char};
use std::sync::OnceLock;

use glib::Object;
use glib::gobject_ffi::GObject;
use glib::translate::{ToGlibPtr, from_glib_full};
use libloading::Library;
use zeroize::Zeroizing;

use crate::domain::Secret;

type Instance = *mut GObject;
type User = unsafe extern "C" fn(i32) -> Instance;
type Name = unsafe extern "C" fn(Instance) -> *const c_char;
type New = unsafe extern "C" fn(Instance, *const c_char) -> Instance;
type Action = unsafe extern "C" fn(Instance);
type Response = unsafe extern "C" fn(Instance, *const c_char);

pub struct Api {
    _core: Library,
    _agent: Library,
    user: User,
    name: Option<Name>,
    new: New,
    initiate: Action,
    cancel: Action,
    response: Response,
}

static API: OnceLock<Result<Api, String>> = OnceLock::new();

pub fn api() -> Result<&'static Api, String> {
    API.get_or_init(Api::load).as_ref().map_err(Clone::clone)
}

impl Api {
    fn load() -> Result<Self, String> {
        unsafe {
            let core =
                Library::new("libpolkit-gobject-1.so.0").map_err(|error| error.to_string())?;
            let agent =
                Library::new("libpolkit-agent-1.so.0").map_err(|error| error.to_string())?;
            Ok(Self {
                user: *core
                    .get::<User>(b"polkit_unix_user_new\0")
                    .map_err(|error| error.to_string())?,
                name: core
                    .get::<Name>(b"polkit_unix_user_get_name\0")
                    .ok()
                    .map(|symbol| *symbol),
                new: *agent
                    .get::<New>(b"polkit_agent_session_new\0")
                    .map_err(|error| error.to_string())?,
                initiate: *agent
                    .get::<Action>(b"polkit_agent_session_initiate\0")
                    .map_err(|error| error.to_string())?,
                cancel: *agent
                    .get::<Action>(b"polkit_agent_session_cancel\0")
                    .map_err(|error| error.to_string())?,
                response: *agent
                    .get::<Response>(b"polkit_agent_session_response\0")
                    .map_err(|error| error.to_string())?,
                _core: core,
                _agent: agent,
            })
        }
    }

    fn identity(&self, uid: u32) -> Result<Object, String> {
        let uid = i32::try_from(uid).map_err(|_| "Usuário inválido")?;
        let pointer = unsafe { (self.user)(uid) };
        if pointer.is_null() {
            return Err("Não foi possível criar a identidade".to_owned());
        }
        Ok(unsafe { from_glib_full(pointer) })
    }

    pub fn username(&self, uid: u32) -> String {
        let fallback = || format!("UID {uid}");
        let Some(name) = self.name else {
            return fallback();
        };
        let Ok(identity) = self.identity(uid) else {
            return fallback();
        };
        let pointer = unsafe { name(identity.to_glib_none().0) };
        if pointer.is_null() {
            return fallback();
        }
        unsafe { CStr::from_ptr(pointer) }
            .to_string_lossy()
            .into_owned()
    }

    pub fn session(&self, uid: u32, cookie: &Secret) -> Result<Object, String> {
        let identity = self.identity(uid)?;
        let cookie = terminated(cookie)?;
        let pointer = unsafe { (self.new)(identity.to_glib_none().0, cookie.as_ptr().cast()) };
        if pointer.is_null() {
            return Err("Não foi possível abrir a autenticação".to_owned());
        }
        Ok(unsafe { from_glib_full(pointer) })
    }

    pub fn initiate(&self, session: &Object) {
        unsafe { (self.initiate)(session.to_glib_none().0) };
    }

    pub fn cancel(&self, session: &Object) {
        unsafe { (self.cancel)(session.to_glib_none().0) };
    }

    pub fn respond(&self, session: &Object, response: &Secret) -> Result<(), String> {
        let response = terminated(response)?;
        unsafe { (self.response)(session.to_glib_none().0, response.as_ptr().cast()) };
        Ok(())
    }
}

fn terminated(secret: &Secret) -> Result<Zeroizing<Vec<u8>>, String> {
    if secret.expose().contains(['\0', '\n', '\r']) {
        return Err("Resposta inválida".to_owned());
    }
    let mut bytes = Zeroizing::new(secret.expose().as_bytes().to_vec());
    bytes.push(0);
    Ok(bytes)
}
