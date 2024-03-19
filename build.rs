use std::env::var;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=purin.dockerfile");

    let manifest_dir = var("CARGO_MANIFEST_DIR").unwrap();
    let src_path = Path::new(&manifest_dir).join("purin.dockerfile");
    let content = read_to_string(src_path).unwrap();

    let mut sharp_marks: String = content
        .chars()
        .filter_map(|c| if c == '#' { Some('#') } else { None })
        .collect();
    sharp_marks.push('#');

    let constant_value = format!(
        "pub const DOCKER_FILE: &str = r{}\"{}\"{};\n",
        &sharp_marks, content, &sharp_marks
    );

    let out_dir = var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("docker_file.rs");

    let mut source = File::create(dest_path).unwrap();
    source.write_all(constant_value.as_bytes()).unwrap();
}
