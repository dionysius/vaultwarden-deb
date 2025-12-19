# Development related notices

Bindings were generated with the following steps:

* Start a debian container via `podman run -it --rm -v ./bindings/:/bindings:z debian bash`
* Run `/bindings/generate_bindings.sh` inside of the container

# Patches on top of the upstream mysql repo:

* Add the possiblity to statically link openssl: https://github.com/mysql/mysql-server/commit/4433b1cbbec301e2141210ee6d19532c4e01d95f
* Revert the removal of the native_password login method: https://github.com/mysql/mysql-server/commit/e1c697a05e4b21f29126dbab3891ae1d8519e113 
