# DEVELOPMENT NOTES


Bindings were generated with the following steps:

* Start a debian container via `podman run -it --rm -v ./:/target:z debian bash`
* Run `/target/generate_bindings.sh` inside of the container

The script generates bindings for the following platforms:

* Linux 64 bit
* Linux 32 bit (different field sizes, compilation fails otherwise due to const checks)
* Windows (MSVC) 64 bit (uses `#[repr(i32)]` instead of `#[repr(u32)]` for enums,
  can simply replace that in the generated linux bindings)
* Windows (MSVC) 32 bit (same as 64 bit windows + 32 bit linux)
