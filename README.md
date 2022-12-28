# deb packaging for vaultwarden

This debian source package builds [vaultwarden](https://github.com/dani-garcia/vaultwarden/) natively on your build environment. No annoying docker! This debian source is managed with [git-buildpackage](https://wiki.debian.org/PackagingWithGit) and is aimed to provide a pretty good quality debian source package (where possible so far). You can find the maintaining command summary in [debian/gbp.conf](debian/gbp.conf).

## Requirements

- Installed `git-buildpackage` `debhelper-compat`
- All build dependencies as defined in [debian/control](debian/control) are installed (it will notify you in the build process as well)
  - [`mk-build-deps`](https://manpages.debian.org/testing/devscripts/mk-build-deps.1.en.html) can help you automate the installation
- Installed `cargo` and its dependencies from your apt sources
  - Don't forget to look into your `*-updates` apt sources for newer versions
  - If your rust/cargo version is not recent enough, this debian source also supports those installed with help of [`rustup`](https://rustup.rs)
    - Requires preloaded `rustup toolchain install <version>` before invoking packaging

## Packaging

- Clone with help of git-buildpackage: `gbp clone https://github.com/dionysius/vaultwarden-deb.git`
- Switch to the folder: `cd vaultwarden-deb`
- Build with help of git-buildpacke: `gbp buildpackage`
  - There are many arguments to fine tune how it is built (see `gbp buildpackage --help` and `dpkg-buildpackage --help`)
  - Mine are usually: `-b` (binary-only, no source files), `-us` (unsigned source package), `-uc` (unsigned .buildinfo and .changes file), `--git-export-dir=<somedir>` (before building the package export the source there)

## Flaws

- I really wanted to use [dh-cargo](https://packages.debian.org/sid/dh-cargo) but I couldn't get it to work (see branch [dh-cargo](https://github.com/dionysius/vaultwarden-deb/tree/dh-cargo/)). Weirdly the compile process happens in `dh_auto_test` instead of `dh_auto_build`. It seems that [dh-cargo is not quite finished and sometimes still opinionated](https://salsa.debian.org/search?search=dh-cargo). There are also packages which ship with their own dh-cargo fork and override it in their debian source (from where I stole this idea and tried to make it work in my branch). It also seems there is not yet a flag for `--features` to be provided in `cargo build` as required by vaultwarden to enable the different database backends.
- I don't have any experience with rust and couldn't quite grasp how I could split the compiling and installing. For now there is one command that does everything in the `dh_auto_install` step.
- For my use case I had to make a compatibility to allow `cargo` to be provided with help of `rustup` since my distro didn't provide a recent enough version. Only using official apt sources would be cleaner.
- This is the first iteration of the package, thus the packaging is in alpha. Contributions are well appreciated.
