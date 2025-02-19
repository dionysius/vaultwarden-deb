# deb packaging for vaultwarden

This debian source package builds [vaultwarden](https://github.com/dani-garcia/vaultwarden/) natively on your build environment with mysql, postgresql and sqlite support. No annoying docker! It is managed with [git-buildpackage](https://wiki.debian.org/PackagingWithGit) and aims to be a pretty good quality debian source package. You can find the maintaining command summary in [debian/gbp.conf](debian/gbp.conf).

You will also need [vaultwarden-web-vault-deb](https://github.com/dionysius/vaultwarden-web-vault-deb).

## Download prebuilt packages

Prebuild deb and src packages are automatically built in [Github Actions](https://github.com/dionysius/vaultwarden-deb/actions) for the latest Ubuntu LTS and Debian stable in various architectures (if applicable).

For manual installation they are available in the [releases section](https://github.com/dionysius/vaultwarden-deb/releases) and you can verify the signatures with this [signing-key](signing-key.pub).

For using apt they are available on [packagecloud](https://packagecloud.io/dionysius/vaultwarden). See their [installation instructions](https://packagecloud.io/dionysius/vaultwarden/install#manual-deb) on how to setup the apt source. Be aware that they use a different signing key. [vaultwarden-web-vault-deb](https://github.com/dionysius/vaultwarden-web-vault-deb) is also automatically uploaded to this repo.

## Requirements

- Installed `git-buildpackage` from your apt
- Installed build dependencies as defined in [debian/control `Build-Depends`](debian/control) (will notify you in the build process otherwise)
  - [`mk-build-deps`](https://manpages.debian.org/testing/devscripts/mk-build-deps.1.en.html) can help you automate the installation
- If `rust`/`cargo` is not recent enough:
  - Don't forget to look into your `*-updates`/`*-backports` apt sources for newer versions
  - This debian source also supports those installed with help of [`rustup`](https://rustup.rs)
    - Requires preloaded `rustup toolchain install <version>` before invoking packaging

## Packaging

- Clone with git-buildpackage: `gbp clone https://github.com/dionysius/vaultwarden-deb.git`
- Switch to the folder: `cd vaultwarden-deb`
- Build with git-buildpackage: `gbp buildpackage`
  - There are many arguments to fine-tune the build (see `gbp buildpackage --help` and `dpkg-buildpackage --help`)
  - Notable options: `-b` (binary-only, no source files), `-us` (unsigned source package), `-uc` (unsigned .buildinfo and .changes file), `--git-export-dir=<somedir>` (before building the package export the source there), `-d` if you need to ignore build-depends (you probably still need them installed from a debian package)

## After Installation

- Configure vaultwarden service environment variables in `/etc/default/vaultwarden` and visit the [vaultwarden wiki](https://github.com/dani-garcia/vaultwarden/wiki) for additional resources

## TODOs

- Automatic notification on new upstream releases. Optimally with automatic PR with those updates
