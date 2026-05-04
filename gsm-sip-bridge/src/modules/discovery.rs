use crate::error::BridgeResult;

const EC20_VENDOR_ID: u16 = 0x2c7c;
const EC20_PRODUCT_ID: u16 = 0x0125;

#[derive(Debug, Clone)]
pub struct DiscoveredModule {
    pub id: String,
    pub serial_port: std::path::PathBuf,
    pub audio_device: String,
    pub usb_serial: String,
}

pub fn derive_module_id(usb_serial: &str) -> String {
    let suffix = if usb_serial.len() >= 6 {
        &usb_serial[usb_serial.len() - 6..]
    } else {
        usb_serial
    };
    format!("ec20-{}", suffix.to_ascii_uppercase())
}

pub fn scan_modules() -> BridgeResult<Vec<DiscoveredModule>> {
    let devices = match rusb::devices() {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(error = %e, "USB device enumeration failed");
            return Ok(Vec::new());
        }
    };

    let mut modules = Vec::new();
    for device in devices.iter() {
        let desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };
        if desc.vendor_id() == EC20_VENDOR_ID && desc.product_id() == EC20_PRODUCT_ID {
            let handle = match device.open() {
                Ok(h) => h,
                Err(_) => continue,
            };
            let serial = handle
                .read_serial_number_string_ascii(&desc)
                .unwrap_or_default();
            if serial.is_empty() {
                continue;
            }
            let id = derive_module_id(&serial);
            tracing::info!(module_id = %id, usb_serial = %serial, "discovered EC20 module");
            modules.push(DiscoveredModule {
                id,
                serial_port: std::path::PathBuf::new(),
                audio_device: String::new(),
                usb_serial: serial,
            });
        }
    }
    Ok(modules)
}
