use std::cell::RefCell;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::ptr;

use zeroize::Zeroizing;

use crate::codes::{
    BUF_ERR, CONV_ERR, ERROR_MSG, PROMPT_ECHO_OFF, PROMPT_ECHO_ON, SUCCESS, TEXT_INFO,
};

#[repr(C)]
struct Message {
    style: c_int,
    text: *const c_char,
}

#[repr(C)]
struct Response {
    text: *mut c_char,
    code: c_int,
}

type Converse = extern "C" fn(c_int, *mut *const Message, *mut *mut Response, *mut c_void) -> c_int;

#[repr(C)]
pub struct Conversation {
    converse: Converse,
    answers: *mut c_void,
}

pub struct Answers {
    pub username: CString,
    pub password: Option<Zeroizing<CString>>,
    pub notes: RefCell<Vec<String>>,
}

impl Conversation {
    pub fn new(answers: &Answers) -> Self {
        Self {
            converse,
            answers: ptr::from_ref(answers).cast_mut().cast(),
        }
    }
}

extern "C" fn converse(
    count: c_int,
    messages: *mut *const Message,
    responses: *mut *mut Response,
    data: *mut c_void,
) -> c_int {
    let Ok(count) = usize::try_from(count) else {
        return CONV_ERR;
    };
    if count == 0 || messages.is_null() || responses.is_null() || data.is_null() {
        return CONV_ERR;
    }
    let answers = unsafe { &*data.cast::<Answers>() };
    let replies = unsafe { libc::calloc(count, size_of::<Response>()) }.cast::<Response>();
    if replies.is_null() {
        return BUF_ERR;
    }
    for index in 0..count {
        let message = unsafe { *messages.add(index) };
        let status = answer(message, answers, unsafe { &mut *replies.add(index) });
        if status != SUCCESS {
            discard(replies, index);
            return status;
        }
    }
    unsafe { *responses = replies };
    SUCCESS
}

fn answer(message: *const Message, answers: &Answers, reply: &mut Response) -> c_int {
    if message.is_null() {
        return CONV_ERR;
    }
    let message = unsafe { &*message };
    let text = match message.style {
        PROMPT_ECHO_OFF => answers
            .password
            .as_ref()
            .map(|password| password.as_c_str()),
        PROMPT_ECHO_ON => Some(answers.username.as_c_str()),
        ERROR_MSG | TEXT_INFO => {
            note(answers, message.text);
            return SUCCESS;
        }
        _ => None,
    };
    let Some(text) = text else {
        return CONV_ERR;
    };
    reply.text = unsafe { libc::strdup(text.as_ptr()) };
    match reply.text.is_null() {
        true => BUF_ERR,
        false => SUCCESS,
    }
}

fn note(answers: &Answers, text: *const c_char) {
    if text.is_null() {
        return;
    }
    let text = unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .trim()
        .to_owned();
    if !text.is_empty() {
        answers.notes.borrow_mut().push(text);
    }
}

fn discard(replies: *mut Response, filled: usize) {
    for index in 0..filled {
        let reply = unsafe { &mut *replies.add(index) };
        if !reply.text.is_null() {
            unsafe {
                libc::explicit_bzero(reply.text.cast(), libc::strlen(reply.text));
                libc::free(reply.text.cast());
            }
        }
    }
    unsafe { libc::free(replies.cast()) };
}
