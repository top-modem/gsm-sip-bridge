use crate::account::Account;
use crate::error::PjsipError;

pub type SlotId = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallState {
    Null,
    Calling,
    Incoming,
    Early,
    Connecting,
    Confirmed,
    Disconnected,
}

pub trait CallStateListener: Send + Sync {
    fn on_call_state(&self, call_id: i32, state: CallState);
    fn on_call_media_state(&self, call_id: i32);
}

pub struct Call {
    #[allow(dead_code)]
    call_id: i32,
    state: CallState,
}

impl Call {
    pub fn make(
        _account: &Account,
        dest_uri: &str,
        _listener: Option<Box<dyn CallStateListener>>,
    ) -> Result<Self, PjsipError> {
        #[cfg(feature = "pjsip-linked")]
        {
            use std::ffi::CString;

            unsafe {
                let uri_cstr = CString::new(dest_uri).map_err(|_| {
                    PjsipError::CallMake("invalid destination URI".into())
                })?;
                let uri = pjsua_sys::pj_str(uri_cstr.as_ptr() as *mut i8);

                let mut call_id: pjsua_sys::pjsua_call_id = -1;
                let status = pjsua_sys::pjsua_call_make_call(
                    _account.account_id(),
                    &uri,
                    std::ptr::null(),
                    std::ptr::null_mut(),
                    std::ptr::null(),
                    &mut call_id,
                );
                if status != crate::error::PJ_SUCCESS {
                    return Err(PjsipError::CallMake(format!(
                        "pjsua_call_make_call returned {status}"
                    )));
                }

                return Ok(Self {
                    call_id,
                    state: CallState::Calling,
                });
            }
        }

        #[cfg(not(feature = "pjsip-linked"))]
        {
            tracing::info!(dest = %dest_uri, "outbound call initiated (stub mode)");
            Ok(Self {
                call_id: 0,
                state: CallState::Calling,
            })
        }
    }

    pub fn hangup(&mut self) -> Result<(), PjsipError> {
        #[cfg(feature = "pjsip-linked")]
        {
            // SAFETY: call_id is valid for the lifetime of this Call object
            unsafe {
                let status = pjsua_sys::pjsua_call_hangup(self.call_id, 200, std::ptr::null(), std::ptr::null());
                if status != crate::error::PJ_SUCCESS {
                    return Err(PjsipError::CallHangup(format!(
                        "pjsua_call_hangup returned {status}"
                    )));
                }
            }
        }

        self.state = CallState::Disconnected;
        Ok(())
    }

    pub fn conf_slot(&self) -> Option<SlotId> {
        #[cfg(feature = "pjsip-linked")]
        {
            // SAFETY: call_id is valid if state is Confirmed
            if self.state == CallState::Confirmed {
                unsafe {
                    let info = std::mem::zeroed::<pjsua_sys::pjsua_call_info>();
                    let status = pjsua_sys::pjsua_call_get_info(self.call_id, &info as *const _ as *mut _);
                    if status == crate::error::PJ_SUCCESS {
                        return Some(info.conf_slot as SlotId);
                    }
                }
            }
            return None;
        }

        #[cfg(not(feature = "pjsip-linked"))]
        {
            if self.state == CallState::Confirmed {
                Some(1)
            } else {
                None
            }
        }
    }

    pub fn state(&self) -> CallState {
        self.state
    }

    pub fn set_state(&mut self, state: CallState) {
        self.state = state;
    }

    pub fn call_id(&self) -> i32 {
        self.call_id
    }
}
