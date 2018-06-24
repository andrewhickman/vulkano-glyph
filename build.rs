fn main() {
    println!("cargo:rerun-if-changed=shaders/frag.glsl");
    println!("cargo:rerun-if-changed=shaders/vert.glsl");
}
