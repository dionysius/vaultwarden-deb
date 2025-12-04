#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#[cfg(feature = "bundled")]
extern crate mysqlclient_src;

#[allow(dead_code)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use self::bindings::*;

// the following type devs are compatibility shims for diesel

#[cfg(not(any(mysql_5_7_x, mariadb_3_1_x, mariadb_3_3_x, mariadb_3_4_x)))]
pub type my_bool = bool;

#[cfg(not(any(mysql_5_7_x, mariadb_3_1_x, mariadb_3_3_x, mariadb_3_4_x)))]
pub const FALSE: my_bool = false;
#[cfg(not(any(mysql_5_7_x, mariadb_3_1_x, mariadb_3_3_x, mariadb_3_4_x)))]
pub const TRUE: my_bool = true;

#[cfg(any(mysql_5_7_x, mariadb_3_1_x, mariadb_3_3_x, mariadb_3_4_x))]
pub const FALSE: my_bool = 0;

#[cfg(any(mysql_5_7_x, mariadb_3_1_x, mariadb_3_3_x, mariadb_3_4_x))]
pub const TRUE: my_bool = 1;

pub const SUPPORTS_MYSQL_SSL_MODE: bool = !cfg!(any(mariadb_3_3_x, mariadb_3_1_x, mariadb_3_4_x));

#[cfg(any(mariadb_3_3_x, mariadb_3_1_x, mariadb_3_4_x))]
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum mysql_ssl_mode {
    SSL_MODE_DISABLED = 1,
    SSL_MODE_PREFERRED = 2,
    SSL_MODE_REQUIRED = 3,
    SSL_MODE_VERIFY_CA = 4,
    SSL_MODE_VERIFY_IDENTITY = 5,
}

#[cfg(any(mariadb_3_3_x, mariadb_3_1_x, mariadb_3_4_x))]
pub mod mysql_option {
    /// that's not supported, do not use it
    pub const MYSQL_OPT_SSL_MODE: crate::bindings::mysql_option =
        crate::bindings::mysql_option::MYSQL_SERVER_PUBLIC_KEY;
    pub const MYSQL_OPT_SSL_CA: crate::bindings::mysql_option =
        crate::bindings::mysql_option::MYSQL_OPT_SSL_CA;
    pub const MYSQL_OPT_SSL_CERT: crate::bindings::mysql_option =
        crate::bindings::mysql_option::MYSQL_OPT_SSL_CERT;
    pub const MYSQL_OPT_SSL_KEY: crate::bindings::mysql_option =
        crate::bindings::mysql_option::MYSQL_OPT_SSL_KEY;
    pub const MYSQL_SET_CHARSET_NAME: crate::bindings::mysql_option =
        crate::bindings::mysql_option::MYSQL_SET_CHARSET_NAME;
}
