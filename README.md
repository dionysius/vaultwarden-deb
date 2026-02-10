# Vaultwarden deb packages

Easy to install and highly configurable debian packages for running [Vaultwarden](https://github.com/dani-garcia/vaultwarden) on your system natively without docker - with mysql, postgresql and sqlite support. Out of the box it can be installed and built on Debian stable, Debian oldstable and latest Ubuntu LTS.

## Installation

The easiest way to install vaultwarden is using the apt repository on [apt.crunchy.run/immich](https://apt.crunchy.run/immich). Installation instructions are available directly on the repository page.

Quick installation:

```bash
sudo apt install curl
curl -fsSL https://apt.crunchy.run/vaultwarden/install.sh | sudo bash -
sudo apt install vaultwarden
```

Alternatively, download prebuilt packages from the [releases section](https://github.com/dionysius/vaultwarden-deb/releases) and verify signatures with the [signing-key](signing-key.pub). Packages are automatically built in [Github Actions](https://github.com/dionysius/vaultwarden-deb/actions). You will also need [vaultwarden-web-vault-deb](https://github.com/dionysius/vaultwarden-web-vault-deb).

## Configuration

After installation, you can configure the service to your liking by editing `/etc/vaultwarden/vaultwarden.env`. Restart the service afterwards using `systemctl restart vaultwarden`. Visit the [Vaultwarden wiki](https://github.com/dani-garcia/vaultwarden/wiki) for additional resources.

## Issues

- [Get in touch](https://github.com/dani-garcia/vaultwarden/wiki#get-in-touch) - For issues with Vaultwarden
- [Issues](https://github.com/dionysius/vaultwarden-deb/issues) and [Discussions](https://github.com/dionysius/vaultwarden-deb/discussions) - For issues with or related to these packages

## Release schedule

This project aims to closely match the releases of upstream. The first release in each minor version series starts as a prerelease with a 7-day waiting period to allow upstream to fix oversights in new features or changes. Subsequent releases follow the same waiting period. After the waiting period has passed, all prereleases are automatically promoted to normal releases including new releases.

## Build source package

This debian source package builds [Vaultwarden](https://github.com/dani-garcia/vaultwarden) natively on your build environment. No annoying docker! It is managed with [git-buildpackage](https://wiki.debian.org/PackagingWithGit) and aims to be a pretty good quality debian source package. You can find the maintaining command summary in [debian/gbp.conf](debian/gbp.conf).

### Requirements

Installed `git-buildpackage` from your apt

Installed build dependencies as defined in [debian/control `Build-Depends`](debian/control) (will notify you in the build process otherwise). [`mk-build-deps`](https://manpages.debian.org/testing/devscripts/mk-build-deps.1.en.html) can help you automate the installation, for example:

```bash
mk-build-deps -i -r debian/control -t "apt-get -o Debug::pkgProblemResolver=yes --no-install-recommends --yes"
```

If `rust`/`cargo` is not recent enough don't forget to look into your `*-updates`/`*-backports` apt sources for newer versions or use [`rustup`](https://rustup.rs) (requires preloaded `rustup toolchain install <version>` before invoking packaging)

### Build package

Clone with git-buildpackage and switch to the folder:

```bash
gbp clone https://github.com/dionysius/vaultwarden-deb.git
cd vaultwarden-deb
```

Build with git-buildpackage - there are many arguments to fine-tune the build (see `gbp buildpackage --help` and `dpkg-buildpackage --help`), notable options: `-b` (binary-only, no source files), `-us` (unsigned source package), `-uc` (unsigned .buildinfo and .changes file), `--git-export-dir=<somedir>` (before building the package export the source there), for example:

```bash
gbp buildpackage -b -us -uc
```

On successful build packages can now be found in the parent directory `ls ../*.deb`.
