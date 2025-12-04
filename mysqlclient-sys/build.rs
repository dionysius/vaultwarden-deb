use std::env;
use std::path::PathBuf;
use std::process::Command;

const PKG_CONFIG_MYSQL_LIB: &str = "mysqlclient";
const PKG_CONFIG_MARIADB_LIB: &str = "libmariadb";
#[cfg(target_env = "msvc")]
const VCPKG_MYSQL_LIB: &str = "libmysql";
#[cfg(target_env = "msvc")]
const VCPKG_MARIADB_LIB: &str = "libmariadb";

fn main() {
    // that's now required as rust doesn't link
    // it automatically anymore
    if env::var("CARGO_CFG_WINDOWS").is_ok() {
        println!("cargo:rustc-link-lib=advapi32");
    }

    if cfg!(feature = "bundled") {
        parse_version("9.3.0");
        return;
    }
    let target = std::env::var("TARGET")
        .expect("Set by cargo")
        .to_ascii_uppercase()
        .replace("-", "_");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_VERSION");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_INCLUDE_DIR");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_INCLUDE_DIR_{target}");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_LIB");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_LIB_DIR");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_LIB_DIR_{target}");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_LIBNAME");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_LIBNAME_{target}");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_STATIC");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_VERSION_{target}");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_LIB_{target}");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_STATIC_{target}");
    if cfg!(feature = "buildtime_bindgen") {
        autogen_bindings(&target);
    }
    let libname = env::var("MYSQLCLIENT_LIBNAME")
        .or_else(|_| env::var(format!("MYSQLCLIENT_LIBNAME_{target}")))
        .unwrap_or("mysqlclient".to_string());
    let link_specifier = if env::var("MYSQLCLIENT_STATIC")
        .or(env::var(format!("MYSQLCLIENT_STATIC_{target}")))
        .is_ok()
    {
        "static="
    } else {
        ""
    };
    if let Ok(lib) = pkg_config::probe_library(PKG_CONFIG_MYSQL_LIB)
        .or_else(|_| pkg_config::probe_library(PKG_CONFIG_MARIADB_LIB))
    {
        // rebuild if users upgraded the mysql library on their machine
        for link in lib.link_paths {
            println!("cargo::rerun-if-changed={}", link.display());
        }
        // pkg_config did everything but the version flags for us
        parse_version(&lib.version);
        return;
    } else if try_vcpkg() {
        // vcpkg did everything for us
        if let Ok(version) =
            env::var("MYSQLCLIENT_VERSION").or(env::var(format!("MYSQLCLIENT_VERSION_{target}")))
        {
            parse_version(&version);
            return;
        }
    } else if let Ok(path) =
        env::var("MYSQLCLIENT_LIB_DIR").or(env::var(format!("MYSQLCLIENT_LIB_DIR_{target}")))
    {
        println!("cargo:rustc-link-search=native={path}");
        println!("cargo:rustc-link-lib={link_specifier}{libname}");
        if let Ok(version) =
            env::var("MYSQLCLIENT_VERSION").or(env::var(format!("MYSQLCLIENT_VERSION_{target}")))
        {
            parse_version(&version);
            return;
        }
    } else if let Some(output) = mysql_config_variable("--libs") {
        let parts = output.split_ascii_whitespace();
        for part in parts {
            if let Some(lib) = part.strip_prefix("-l") {
                println!("cargo:rustc-link-lib={link_specifier}{lib}");
            } else if let Some(path) = part.strip_prefix("-L") {
                println!("cargo:rustc-link-search=native={path}");
            } else if let Some(path) = part.strip_prefix("-R") {
                println!("cargo:rustc-link-arg=-Wl,-R{path}");
            } else {
                panic!("Unexpected output from mysql_config: `{output}`");
            }
        }

        if let Some(version) = mysql_config_variable("--version") {
            parse_version(&version);
            return;
        }
    }
    panic!(
        "Did not find a compatible version of libmysqlclient.\n\
            Ensure that you installed one and teached mysqlclient-sys how to find it\n\
            You have the following options for that:\n\
            \n\
            * Use `pkg_config` to automatically detect the right location\n\
            * Use vcpkg to automatically detect the right location. \n\
              You also need to set `MYSQLCLIENT_VERSION` to specify which\n\
              version of libmysqlclient you are using\n\
            * Set the `MYSQLCLIENT_LIB_DIR` and `MYSQLCLIENT_VERSION` environment \n\
              variables to point the compiler to the right directory and specify \n\
              which version is used\n\
            * Make the `mysql_config` binary avaible in the environment that invokes\n\
              the compiler"
    );
}

fn mysql_config_variable(var_name: &str) -> Option<String> {
    Command::new("mysql_config")
        .arg(var_name)
        .output()
        .into_iter()
        .filter(|output| output.status.success())
        .flat_map(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_string())
        .next()
}

#[derive(Clone, Copy, Debug)]
enum MysqlVersion {
    Mysql5,
    Mysql80,
    Mysql81,
    Mysql82,
    Mysql83,
    Mysql84,
    Mysql90,
    Mysql91,
    Mysql92,
    Mysql93,
    MariaDb31,
    MariaDb33,
    MariaDb34,
}

impl MysqlVersion {
    const ALL: &'static [Self] = &[
        Self::Mysql5,
        Self::Mysql80,
        Self::Mysql81,
        Self::Mysql82,
        Self::Mysql83,
        Self::Mysql84,
        Self::Mysql90,
        Self::Mysql91,
        Self::Mysql92,
        Self::Mysql93,
        Self::MariaDb31,
        Self::MariaDb33,
        Self::MariaDb34,
    ];

    fn as_cfg(&self) -> &'static str {
        match self {
            MysqlVersion::Mysql5 => "mysql_5_7_x",
            MysqlVersion::Mysql80 => "mysql_8_0_x",
            MysqlVersion::Mysql81 => "mysql_8_1_x",
            MysqlVersion::Mysql82 => "mysql_8_2_x",
            MysqlVersion::Mysql83 => "mysql_8_3_x",
            MysqlVersion::Mysql84 => "mysql_8_4_x",
            MysqlVersion::Mysql90 => "mysql_9_0_x",
            MysqlVersion::Mysql91 => "mysql_9_1_x",
            MysqlVersion::Mysql92 => "mysql_9_2_x",
            MysqlVersion::Mysql93 => "mysql_9_3_x",
            MysqlVersion::MariaDb31 => "mariadb_3_1_x",
            MysqlVersion::MariaDb33 => "mariadb_3_3_x",
            MysqlVersion::MariaDb34 => "mariadb_3_4_x",
        }
    }

    fn to_binding_version(&self) -> &'static str {
        match self {
            MysqlVersion::Mysql5 => "5_7_42",
            MysqlVersion::Mysql80 => "8_0_39",
            MysqlVersion::Mysql81 => "8_1_0",
            MysqlVersion::Mysql82 => "8_2_0",
            MysqlVersion::Mysql83 => "8_3_0",
            MysqlVersion::Mysql84 => "8_4_3",
            MysqlVersion::Mysql90 => "9_0_1",
            MysqlVersion::Mysql91 => "9_1_0",
            MysqlVersion::Mysql92 => "9_2_0",
            MysqlVersion::Mysql93 => "9_3_0",
            MysqlVersion::MariaDb31 => "mariadb_3_1_27",
            MysqlVersion::MariaDb33 => "mariadb_3_3_14",
            MysqlVersion::MariaDb34 => "mariadb_3_4_4",
        }
    }

    fn parse_version(version: &str) -> Option<Self> {
        // ubuntu/debian packages use the following package versions:
        // libmysqlclient20 -> 5.7.x
        // libmysqlclient21 -> 8.0.x
        // libmysqlclient22 -> 8.2.x
        // libmysqlclient23 -> 8.3.0
        // libmysqlclient24 -> 8.4.0 or 9.0 or 9.1 or 9.2
        // Linux version becomes the full SONAME like 21.3.2 but MacOS is just the
        // major version.
        //
        // For libmariadb the mapping is a bit more complicated
        //
        // Mappings can be reconstructed by checking
        // the mariadb repo here: https://github.com/MariaDB/server/
        // for each relevant tag and look at the linked libmariadb client submodule.
        // In the linked submodule the CMakeLists.txt file contains the version
        //
        // * mariadb version 10.2.x -> 3.0.x/3.1.x (3.0 is compatible with 3.1)
        // * mariadb version 10.3.x -> 3.0.x/3.1.x (3.0 is compatible with 3.1)
        // * mariadb version 10.4.x -> 3.1.x
        // * mariadb version 10.5.x -> 3.1.x
        // * mariadb version 10.6.x -> 3.2.x/3.3.x (3.2 is compatible with 3.3)
        // * mariadb version 10.7.x -> 3.2.x/3.3.x (3.2 is compatible with 3.3)
        // * mariadb version 10.8.x -> 3.3.x
        // * mariadb version 10.9.x -> 3.3.x
        // * mariadb version 10.10.x -> 3.3.x
        // * mariadb version 10.11.x -> 3.3.x
        // * mariadb version 11.0.x -> 3.3.x
        // * mariadb version 11.1.x -> 3.3.x
        // * mariadb version 11.2.x -> 3.3.x
        // * mariadb version 11.3.x -> 3.3.x
        // * mariadb version 11.4.x -> 3.4.x (11.4.0 references 3.3.x, but I believe that might be a mistake)
        // * mariadb version 11.5.x -> 3.4.x
        // * mariadb version 11.6.x -> 3.4.x
        // * mariadb version 11.7.x -> 3.4.x
        // * mariadb version 11.8.x -> 3.4.x
        if version.starts_with("5.7") || version.starts_with("20.") || version == "20" {
            Some(Self::Mysql5)
        } else if version.starts_with("8.0") || version.starts_with("21.") || version == "21" {
            Some(Self::Mysql80)
        } else if version.starts_with("8.1") {
            Some(Self::Mysql81)
        } else if version.starts_with("8.2") || version.starts_with("22.") || version == "22" {
            Some(Self::Mysql82)
        } else if version.starts_with("8.3") || version.starts_with("23.") || version == "23" {
            Some(Self::Mysql83)
        } else if version.starts_with("8.4") || version.starts_with("24.0") || version == "24" {
            Some(Self::Mysql84)
        } else if version.starts_with("9.0") {
            Some(Self::Mysql90)
        } else if version.starts_with("9.1") {
            Some(Self::Mysql91)
        } else if version.starts_with("9.2") || version.starts_with("24.1") {
            Some(Self::Mysql92)
        } else if version.starts_with("9.3") {
            Some(Self::Mysql93)
        } else if version.starts_with("3.1") || match_semver(">=10.2.0, <10.6.0", version) {
            Some(Self::MariaDb31)
        } else if version.starts_with("3.2")
            || version.starts_with("3.3")
            || match_semver(">=10.6.0, <11.4.0", version)
        {
            Some(Self::MariaDb33)
        } else if version.starts_with("3.4") || version.starts_with("11") {
            Some(Self::MariaDb34)
        } else {
            None
        }
    }

    fn as_display_version(&self) -> &'static str {
        match self {
            MysqlVersion::Mysql5 => "Mysql 5.7.x",
            MysqlVersion::Mysql80 => "Mysql 8.0.x",
            MysqlVersion::Mysql81 => "Mysql 8.1.x",
            MysqlVersion::Mysql82 => "Mysql 8.2.x",
            MysqlVersion::Mysql83 => "Mysql 8.3.x",
            MysqlVersion::Mysql84 => "Mysql 8.4.x",
            MysqlVersion::Mysql90 => "Mysql 9.0.x",
            MysqlVersion::Mysql91 => "Mysql 9.1.x",
            MysqlVersion::Mysql92 => "Mysql 9.2.x",
            MysqlVersion::Mysql93 => "Mysql 9.3.x",
            MysqlVersion::MariaDb31 => "MariaDB 3.1.x",
            MysqlVersion::MariaDb33 => "MariaDB 3.3.x",
            MysqlVersion::MariaDb34 => "MariaDB 3.4.x",
        }
    }
}

fn match_semver(version_req: &str, version: &str) -> bool {
    use semver::{Version, VersionReq};

    let Ok(ver) = Version::parse(&version) else {
        return false;
    };
    let req = VersionReq::parse(&version_req).expect("Version matching string is invalid");
    req.matches(&ver)
}

fn parse_version(version_str: &str) {
    for v in MysqlVersion::ALL {
        println!("cargo::rustc-check-cfg=cfg({})", v.as_cfg());
    }
    let version = MysqlVersion::parse_version(version_str);

    let is_windows = std::env::var("CARGO_CFG_WINDOWS").is_ok();
    let ptr_size = std::env::var("CARGO_CFG_TARGET_POINTER_WIDTH").expect("Set by cargo");
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").expect("Set by cargo");
    let out_dir = std::env::var("OUT_DIR").expect("Set by cargo");
    let mut bindings_target = PathBuf::from(out_dir);
    bindings_target.push("bindings.rs");

    if let Some(version) = version {
        println!("cargo:rustc-cfg={}", version.as_cfg());
        if cfg!(feature = "buildtime_bindgen") {
            return;
        }
        let os = if is_windows { "windows" } else { "linux" };
        let arch = match (ptr_size.as_str(), target_arch.as_str()) {
            ("32", "arm") => "arm",
            ("32", _) => "i686",
            ("64", _) => "x86_64",
            (s, _) => panic!(
                "Pointer size: `{s}` is not supported by mysqlclient-sys. \
                 Consider using the `buildtime_bindgen` feature to generate matching bindings at build time"
            ),
        };
        let bindings_path = format!("bindings_{}_{arch}_{os}.rs", version.to_binding_version());
        let root = std::env::var("CARGO_MANIFEST_DIR").expect("Set by cargo");
        let mut bindings = PathBuf::from(root);
        bindings.push("bindings");
        bindings.push(bindings_path);
        std::fs::copy(bindings, bindings_target).unwrap();
    } else {
        let possible_versions = MysqlVersion::ALL
            .iter()
            .map(|v| v.as_display_version())
            .collect::<Vec<_>>();
        panic!("`{version_str}` is not supported by the mysqlclient-sys crate. \
                Any of the following versions is supported: {possible_versions:?}. \
                Consider using the `buildtime_bindgen` feature to generate matching bindings at build time. \n\
                If you set the version via the `MYSQLCLIENT_VERSION` variable make sure that it is a valid semver\
                version like `8.0.32`");
    }
}

#[cfg(target_env = "msvc")]
fn try_vcpkg() -> bool {
    if vcpkg::find_package(VCPKG_MYSQL_LIB).is_ok() {
        return true;
    } else if vcpkg::find_package(VCPKG_MARIADB_LIB).is_ok() {
        return true;
    }
    false
}

#[cfg(not(target_env = "msvc"))]
fn try_vcpkg() -> bool {
    false
}

#[cfg(not(feature = "buildtime_bindgen"))]
fn autogen_bindings(_target: &str) {}

#[cfg(feature = "buildtime_bindgen")]
fn autogen_bindings(target: &str) {
    // if you update the options here you also need to
    // update the bindgen command in `DEVELOPMENT.md`
    // and regenerate the bundled bindings with the new options
    let mut builder = include!("src/make_bindings.rs")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    if let Ok(lib) = pkg_config::probe_library(PKG_CONFIG_MYSQL_LIB)
        .or_else(|_| pkg_config::probe_library(PKG_CONFIG_MARIADB_LIB))
    {
        for include in lib.include_paths {
            builder = builder.clang_arg(format!("-I{}", include.display()));
        }
    } else if let Ok(path) = env::var("MYSQLCLIENT_INCLUDE_DIR")
        .or_else(|_| env::var(format!("MYSQLCLIENT_INCLUDE_DIR_{target}")))
    {
        builder = builder.clang_arg(format!("-I{path}"));
    } else if let Some(include) = mysql_config_variable("--include") {
        builder = builder.clang_arg(include);
    } else {
        #[cfg(target_env = "msvc")]
        if let Ok(lib) =
            vcpkg::find_package(VCPKG_MYSQL_LIB).or_else(|_| vcpkg::find_package(VCPKG_MARIADB_LIB))
        {
            for include in lib.include_paths {
                builder = builder.clang_arg(format!("-I{}\\mysql", include.display()));
            }
        }
    }

    let bindings = builder
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = std::path::PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
