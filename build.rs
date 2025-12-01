//! Build script for Hi Happy Garden Rust firmware

fn main() {
    println!("cargo:rerun-if-changed=memory.x");
}
