extern crate pq_sys;

#[cfg(not(feature = "bundled_without_openssl"))]
#[test]
fn test_ssl_init() {
    unsafe {
        pq_sys::PQinitSSL(1);
    }
}
