fn main() {
    let ac = autocfg::new();
    ac.emit_path_cfg("std::io::Read::read_vectored", "vectored");

    autocfg::rerun_path("build.rs");
}
