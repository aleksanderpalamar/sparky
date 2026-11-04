use std::ffi::{c_char, c_int, c_void};
use std::sync::OnceLock;

use libloading::Library;

use crate::conversation::Conversation;

const LIBRARY: &str = "libpam.so.0";

pub type Handle = *mut c_void;
type Start =
    unsafe extern "C" fn(*const c_char, *const c_char, *const Conversation, *mut Handle) -> c_int;
type StartIn = unsafe extern "C" fn(
    *const c_char,
    *const c_char,
    *const Conversation,
    *const c_char,
    *mut Handle,
) -> c_int;
type Step = unsafe extern "C" fn(Handle, c_int) -> c_int;
type SetItem = unsafe extern "C" fn(Handle, c_int, *const c_void) -> c_int;
type PutEnv = unsafe extern "C" fn(Handle, *const c_char) -> c_int;
type GetEnvList = unsafe extern "C" fn(Handle) -> *mut *mut c_char;
type StrError = unsafe extern "C" fn(Handle, c_int) -> *const c_char;

pub struct Api {
    _library: Library,
    pub start: Start,
    pub start_in: Option<StartIn>,
    pub authenticate: Step,
    pub account: Step,
    pub set_credentials: Step,
    pub open_session: Step,
    pub close_session: Step,
    pub end: Step,
    pub set_item: SetItem,
    pub put_env: PutEnv,
    pub env_list: GetEnvList,
    pub describe: StrError,
}

static API: OnceLock<Result<Api, String>> = OnceLock::new();

pub fn api() -> Result<&'static Api, String> {
    API.get_or_init(load).as_ref().map_err(Clone::clone)
}

fn load() -> Result<Api, String> {
    let library =
        unsafe { Library::new(LIBRARY) }.map_err(|error| format!("{LIBRARY}: {error}"))?;
    let api = unsafe {
        Api {
            start: *library.get::<Start>(b"pam_start\0").map_err(missing)?,
            start_in: library
                .get::<StartIn>(b"pam_start_confdir\0")
                .ok()
                .map(|symbol| *symbol),
            authenticate: *library
                .get::<Step>(b"pam_authenticate\0")
                .map_err(missing)?,
            account: *library.get::<Step>(b"pam_acct_mgmt\0").map_err(missing)?,
            set_credentials: *library.get::<Step>(b"pam_setcred\0").map_err(missing)?,
            open_session: *library
                .get::<Step>(b"pam_open_session\0")
                .map_err(missing)?,
            close_session: *library
                .get::<Step>(b"pam_close_session\0")
                .map_err(missing)?,
            end: *library.get::<Step>(b"pam_end\0").map_err(missing)?,
            set_item: *library.get::<SetItem>(b"pam_set_item\0").map_err(missing)?,
            put_env: *library.get::<PutEnv>(b"pam_putenv\0").map_err(missing)?,
            env_list: *library
                .get::<GetEnvList>(b"pam_getenvlist\0")
                .map_err(missing)?,
            describe: *library
                .get::<StrError>(b"pam_strerror\0")
                .map_err(missing)?,
            _library: library,
        }
    };
    Ok(api)
}

fn missing(error: libloading::Error) -> String {
    format!("{LIBRARY} incompleta: {error}")
}
