fn main() {
    println!("cargo:rerun-if-changed=shader/frag.glsl");
    println!("cargo:rerun-if-changed=shader/vert.glsl");
}
