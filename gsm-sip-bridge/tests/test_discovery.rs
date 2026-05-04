use gsm_sip_bridge::modules::discovery::derive_module_id;

#[test]
fn test_derive_module_id_from_full_serial() {
    let id = derive_module_id("1234567890ABCDEF");
    assert_eq!(id, "ec20-ABCDEF");
}

#[test]
fn test_derive_module_id_short_serial() {
    let id = derive_module_id("AB");
    assert_eq!(id, "ec20-AB");
}

#[test]
fn test_derive_module_id_exact_six() {
    let id = derive_module_id("a1b2c3");
    assert_eq!(id, "ec20-A1B2C3");
}

#[test]
fn test_derive_module_id_uppercase() {
    let id = derive_module_id("000000abcdef");
    assert_eq!(id, "ec20-ABCDEF");
}
