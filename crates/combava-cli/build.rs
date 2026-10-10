//! Recompile le binaire quand un template embarqué change : `include_dir!` ne
//! le signale pas à cargo sur la version stable de Rust.

fn main() {
    println!("cargo::rerun-if-changed=../../templates");
}
