fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // Full bindgen invocation will be implemented in T025.
    // For now, emit a placeholder so the crate compiles.
}
