use std::env::var;
use std::fs::copy;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=purin.dockerfile");

    let manifest_dir = var("CARGO_MANIFEST_DIR").unwrap();
    let src_path = Path::new(&manifest_dir).join("purin.dockerfile");

    let out_dir = var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("purin.dockerfile");

    copy(&src_path, &dest_path).unwrap();

    println!(
        "{} {}",
        src_path.to_str().unwrap(),
        dest_path.to_str().unwrap()
    );

    std::fs::File::create(dest_path).unwrap();
}
