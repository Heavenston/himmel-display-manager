use std::ptr::null_mut;
use std::ffi::{ c_void, CStr, CString };
use std::mem;
use libc::{ c_int, calloc, size_t, strdup, free };

use pam_sys::types::{
    PamConversation,
    PamMessage,
    PamResponse,
    PamHandle,
    PamReturnCode,
    PamMessageStyle,
    PamFlag,
};
use pam_sys::wrapped as pms;

/// Stands for "Check Pam Return Code"
#[must_use]
fn cprc(code: PamReturnCode) -> Result<(), PamReturnCode> {
    if code != PamReturnCode::SUCCESS {
        Err(code)
    }
    else {
        Ok(())
    }
}

pub(crate) extern "C" fn pam_conv(
    num_msg: c_int,
    in_msg:  *mut *mut PamMessage,
    out_resp: *mut *mut PamResponse,
    appdata_ptr: *mut c_void,
) -> c_int { unsafe { // Unsafe is here to avoid changing the signature
    let resp_ptr = calloc(num_msg as usize, mem::size_of::<PamResponse>() as size_t) as *mut PamResponse;
    if resp_ptr.is_null() {
        return PamReturnCode::BUF_ERR as c_int;
    }
    let resp = &mut *resp_ptr;

    let (username, password) = &*(appdata_ptr as *const (CString, CString));
    let mut result: PamReturnCode = PamReturnCode::SUCCESS;

    for i in 0..num_msg as isize {
        let current_msg  = &mut **in_msg.offset(i);
        let msg = CStr::from_ptr(current_msg.msg);

        // match on msg_style
        match PamMessageStyle::from(current_msg.msg_style) {
            PamMessageStyle::PROMPT_ECHO_ON => {
                resp.resp = strdup(username.as_ptr());
            }
            PamMessageStyle::PROMPT_ECHO_OFF => {
                resp.resp = strdup(password.as_ptr());
            }
            PamMessageStyle::TEXT_INFO => {
                println!("INFO: {}", msg.to_str().unwrap());
            }
            PamMessageStyle::ERROR_MSG => {
                println!("ERROR: {}", msg.to_str().unwrap());
                result = PamReturnCode::CONV_ERR;
            }
        }

        if result != PamReturnCode::SUCCESS {
            break;
        }
    }

    // free allocated memory if an error occured
    if result != PamReturnCode::SUCCESS {
        free(resp_ptr as *mut c_void);
    } else {
        *out_resp = resp_ptr;
    }

    result as c_int
} }

pub struct Author {
    handle: *mut PamHandle,
    data: Box<(CString, CString)>,
}

impl Author {
    pub fn new() -> Self {
        let mut handle = null_mut();
        let mut data = Box::new((
            CString::new("").unwrap(),
            CString::new("").unwrap(),
        ));

        pms::start("system-auth", None, &PamConversation {
            conv: Some(pam_conv),
            data_ptr: (&mut *data) as *mut _ as *mut c_void,
        }, &mut handle);
        
        Self { handle, data }
    }

    pub fn set_username(&mut self, username: impl Into<Vec<u8>>) -> &mut Self {
        self.data.0 = CString::new(username.into()).expect("CString::new failed");
        self
    }

    pub fn set_password(&mut self, password: impl Into<Vec<u8>>) -> &mut Self {
        self.data.1 = CString::new(password.into()).expect("CString::new failed");
        self
    }

    pub fn open_session(&mut self) -> Result<(), PamReturnCode> {
        let handle = unsafe { &mut *self.handle };
        cprc(pms::authenticate(handle, PamFlag::NONE))?;
        cprc(pms::acct_mgmt(   handle, PamFlag::NONE))?;
        cprc(pms::setcred(     handle, PamFlag::ESTABLISH_CRED))?;
        let session_r = cprc(pms::open_session(handle, PamFlag::NONE));
        match session_r {
            Ok(()) => (),
            Err(e) => {
                cprc(pms::setcred(handle, PamFlag::DELETE_CRED))?;
                return Err(e);
            },
        }

        Ok(())
    }
}

unsafe impl Send for Author {}
unsafe impl Sync for Author {}
