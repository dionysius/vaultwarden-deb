// if you update this command you also need to update the
// command in `bindings/generate_bindings.sh`

bindgen::Builder::default()
// The input header we would like to generate
// bindings for.
    .header("bindings/wrapper.h")
    .allowlist_function("mysql.*")
    .allowlist_function("mariadb.*")
    .allowlist_type("MYSQL.*")
    .allowlist_type("MARIADB.*")
    .allowlist_type("mysql.*")
    .allowlist_type("mariadb.*")
    .allowlist_var("MYSQL.*")
    .allowlist_var("MARIADB.*")
    .default_enum_style(bindgen::EnumVariation::Rust {
        non_exhaustive: true,
    })
