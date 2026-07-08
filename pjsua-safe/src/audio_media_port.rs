use crate::call::SlotId;
use crate::error::PjsipError;
#[cfg(feature = "pjsip-linked")]
use pjsua_sys::pj_status_t;
#[cfg(feature = "pjsip-linked")]
use std::sync::Mutex;

/// Serializes concurrent calls to `pjsua_conf_add_port` from module init
/// threads. PJSIP's conference bridge is not fully re-entrant when adding
/// new ports while the sound device thread is active.
#[cfg(feature = "pjsip-linked")]
static CONF_BRIDGE_LOCK: Mutex<()> = Mutex::new(());

pub trait AudioMediaPort: Send {
    fn read_frame(&mut self, buf: &mut [i16]);
    fn write_frame(&mut self, buf: &[i16]);
}

pub struct MediaPortHandle {
    slot_id: SlotId,
}

impl MediaPortHandle {
    pub fn register_to_conf_bridge(port: Box<dyn AudioMediaPort>) -> Result<Self, PjsipError> {
        #[cfg(feature = "pjsip-linked")]
        {
            crate::endpoint::ensure_pjsip_thread();

            use std::ffi::CString;
            unsafe {
                let pool = pjsua_sys::pjsua_pool_create(
                    b"gsm-port\0".as_ptr() as *const std::os::raw::c_char,
                    512,
                    512,
                );
                if pool.is_null() {
                    return Err(PjsipError::MediaPort("pool creation failed".into()));
                }

                // Double-box to store the fat trait-object pointer as a thin pointer.
                let port_ptr = Box::into_raw(Box::new(port)) as *mut std::os::raw::c_void;

                let mut slot: pjsua_sys::pjsua_conf_port_id = -1;

                // Hold the serialization lock while setting up and adding the
                // port, so PJSIP's conference bridge is not racing with another
                // module's port registration from its own blocking thread.
                //
                // The mutex is a standard-library Mutex, so we hold it across
                // the FFI call. We release it as soon as conf_add_port returns.
                let _bridge_lock = CONF_BRIDGE_LOCK.lock().unwrap();

                // Allocate the pjmedia_port from PJSIP's pool so it outlives
                // this function. PJSIP's conf bridge stores a POINTER to the
                // port (not a copy), so a stack allocation would dangle.
                let media_port = &mut *(pjsua_sys::pj_pool_calloc(
                    pool,
                    1,
                    std::mem::size_of::<pjsua_sys::pjmedia_port>() as pjsua_sys::pj_size_t,
                ) as *mut pjsua_sys::pjmedia_port);

                // Leak the CString so it lives for the entire program lifetime.
                // PJSIP's pjsua_conf_add_port uses pj_strdup to deep-copy the
                // name, but keeping the original alive avoids any potential
                // stale-pointer issue.
                let name = CString::new("gsm-media").unwrap().into_raw();

                media_port.info.name.ptr = name as *mut std::os::raw::c_char;
                media_port.info.name.slen = 9;

                media_port.info.signature = 0xBEEF;

                media_port.info.dir =
                    pjsua_sys::pjmedia_dir_PJMEDIA_DIR_CAPTURE_PLAYBACK as pjsua_sys::pjmedia_dir;

                media_port.info.fmt.id = pjsua_sys::pjmedia_format_id_PJMEDIA_FORMAT_PCM;
                media_port.info.fmt.type_ =
                    pjsua_sys::pjmedia_type_PJMEDIA_TYPE_AUDIO as pjsua_sys::pjmedia_type;
                media_port.info.fmt.detail_type =
                    pjsua_sys::pjmedia_format_detail_type_PJMEDIA_FORMAT_DETAIL_AUDIO
                        as pjsua_sys::pjmedia_format_detail_type;
                media_port.info.fmt.det.aud.clock_rate = 8000;
                media_port.info.fmt.det.aud.channel_count = 1;
                media_port.info.fmt.det.aud.bits_per_sample = 16;
                media_port.info.fmt.det.aud.frame_time_usec = 20000;
                media_port.info.fmt.det.aud.avg_bps = 128000;
                media_port.info.fmt.det.aud.max_bps = 128000;

                media_port.port_data.pdata = port_ptr as *mut std::os::raw::c_void;

                media_port.get_frame = Some(get_frame_callback);
                media_port.put_frame = Some(put_frame_callback);
                media_port.on_destroy = Some(on_destroy_callback);

                let status = pjsua_sys::pjsua_conf_add_port(pool, media_port, &mut slot);
                // _bridge_lock dropped here — other modules can register now.
                if status != crate::error::PJ_SUCCESS {
                    let _ = Box::from_raw(port_ptr);
                    return Err(PjsipError::MediaPort(format!(
                        "pjsua_conf_add_port returned {status}"
                    )));
                }

                tracing::info!(slot, "GSM media port registered to conference bridge");
                Ok(Self { slot_id: slot })
            }
        }

        #[cfg(not(feature = "pjsip-linked"))]
        {
            let _ = port;
            tracing::debug!("media port registered to conference bridge (stub mode)");
            Ok(Self { slot_id: 1 })
        }
    }

    pub fn slot_id(&self) -> SlotId {
        self.slot_id
    }

    pub fn connect_to(&self, _dest_slot: SlotId) -> Result<(), PjsipError> {
        #[cfg(feature = "pjsip-linked")]
        {
            unsafe // SAFETY: PJSIP initialized; slot IDs valid conference bridge ports for connect
            {
                let status = pjsua_sys::pjsua_conf_connect(self.slot_id, _dest_slot);
                if status != crate::error::PJ_SUCCESS {
                    return Err(PjsipError::MediaPort(format!(
                        "pjsua_conf_connect returned {status}"
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn disconnect_from(&self, _dest_slot: SlotId) -> Result<(), PjsipError> {
        #[cfg(feature = "pjsip-linked")]
        {
            unsafe // SAFETY: PJSIP initialized; slot IDs valid conference bridge ports for disconnect
            {
                let status = pjsua_sys::pjsua_conf_disconnect(self.slot_id, _dest_slot);
                if status != crate::error::PJ_SUCCESS {
                    return Err(PjsipError::MediaPort(format!(
                        "pjsua_conf_disconnect returned {status}"
                    )));
                }
            }
        }
        Ok(())
    }
}

#[cfg(feature = "pjsip-linked")]
unsafe extern "C" fn get_frame_callback(
    this_port: *mut pjsua_sys::pjmedia_port,
    frame: *mut pjsua_sys::pjmedia_frame,
) -> pj_status_t {
    let port_ptr = (*this_port).port_data.pdata;
    if port_ptr.is_null() {
        return crate::error::PJ_SUCCESS;
    }
    let port: &mut Box<dyn AudioMediaPort> = &mut *(port_ptr as *mut Box<dyn AudioMediaPort>);

    let buf = (*frame).buf;
    let size = (*frame).size;
    if buf.is_null() || size < 2 {
        return crate::error::PJ_SUCCESS;
    }
    let samples = (size / 2) as usize;
    let slice = std::slice::from_raw_parts_mut(buf as *mut i16, samples);
    port.read_frame(slice);
    (*frame).type_ = pjsua_sys::pjmedia_frame_type_PJMEDIA_FRAME_TYPE_AUDIO;
    (*frame).size = (samples * 2) as pjsua_sys::pj_size_t;
    crate::error::PJ_SUCCESS
}

#[cfg(feature = "pjsip-linked")]
unsafe extern "C" fn put_frame_callback(
    this_port: *mut pjsua_sys::pjmedia_port,
    frame: *mut pjsua_sys::pjmedia_frame,
) -> pj_status_t {
    let port_ptr = (*this_port).port_data.pdata;
    if port_ptr.is_null() {
        return crate::error::PJ_SUCCESS;
    }
    let port: &mut Box<dyn AudioMediaPort> = &mut *(port_ptr as *mut Box<dyn AudioMediaPort>);

    let buf = (*frame).buf;
    let size = (*frame).size;
    if buf.is_null() || size < 2 {
        return crate::error::PJ_SUCCESS;
    }
    let samples = (size / 2) as usize;
    let slice = std::slice::from_raw_parts(buf as *const i16, samples);
    port.write_frame(slice);
    crate::error::PJ_SUCCESS
}

#[cfg(feature = "pjsip-linked")]
unsafe extern "C" fn on_destroy_callback(this_port: *mut pjsua_sys::pjmedia_port) -> pj_status_t {
    let port_ptr = (*this_port).port_data.pdata;
    if !port_ptr.is_null() {
        let _ = Box::from_raw(port_ptr as *mut Box<dyn AudioMediaPort>);
        (*this_port).port_data.pdata = std::ptr::null_mut();
    }
    crate::error::PJ_SUCCESS
}
