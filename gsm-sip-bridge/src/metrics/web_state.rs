use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct WebSlotInfo {
    pub slot: u32,
    pub state: String,
    pub imei: String,
    pub phone: String,
    pub network: String,
    pub active_call: bool,
}

pub type SharedSlots = Arc<RwLock<Vec<WebSlotInfo>>>;
