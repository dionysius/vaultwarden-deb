fn main() {
    if let Some(true) = version_check::is_feature_flaggable() {
        println!("cargo:rustc-cfg=nightly");
    }

    if let Some(true) = version_check::is_min_version("1.56.0") {
        println!("cargo:rustc-cfg=const_fn_transmute");
    }
}
