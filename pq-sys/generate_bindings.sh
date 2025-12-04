#!/usr/bin/env bash

DEBIAN_FRONTEND=noninteractive
export DEBIAN_FRONTEND

set +ex

apt update
apt install -y binutils xz-utils curl libclang-dev gcc mingw-w64 gcc-i686-linux-gnu clang gcc-arm-linux-gnueabi libpq-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal -y -c rustfmt
. "/root/.cargo/env"
cargo install bindgen-cli@0.72.0

function bindgen_common() {
    bindgen --allowlist-function "PQ.*" \
        --allowlist-function "lo_.*" \
        --allowlist-function "pg_.*" \
        --opaque-type "FILE" \
        --blocklist-type "FILE" \
        --raw-line "use libc::FILE;" \
        --allowlist-type "Oid" \
        --allowlist-type "ConnStatusType" \
        --allowlist-type "Postgres.*" \
        --allowlist-type "pg.*" \
        --allowlist-type "PG.*" \
        --allowlist-type "PQ.*" \
        --allowlist-type "pq.*" \
        --allowlist-var "PG_.*" \
        --allowlist-var "LIBPQ.*" \
        --allowlist-var "PQ.*" \
        --rustified-enum ".*" \
        --no-derive-default \
        --generate "functions,types,vars,methods,constructors,destructors" \
        /target/wrapper.h \
        -- -I /target/pq-src/source/src/interfaces/libpq/ -I /target/pq-src/source/src/include/ -I /target/pq-src/additional_include/ $@
}

bindgen_common >/target/src/bindings_linux.rs
bindgen_common -target i686-unknown-linux-gnu >/target/src/bindings_linux_32.rs
bindgen_common -target x86_64-pc-windows-gnu | sed -e 's/#\[repr(u32)\]/#\[repr(i32)\]/g' >/target/src/bindings_windows.rs
bindgen_common -target i686-pc-windows-gnu | sed -e 's/#\[repr(u32)\]/#\[repr(i32)\]/g' >/target/src/bindings_windows_32.rs
