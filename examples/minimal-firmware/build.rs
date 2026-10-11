fn main() {
    // Make the example's memory.x visible to the linker alongside cortex-m-rt's link.x.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set");
    println!("cargo:rustc-link-search={manifest_dir}");
    println!("cargo:rerun-if-changed=memory.x");
}
