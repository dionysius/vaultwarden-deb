# totp-lite Changelog

## 2.0.1 (2023-10-30)

#### Changed

- Switch to `sha1` to `sha-1`, which was deprecated. 

## 2.0.0 (2022-05-19)

#### Changed

- Bump cryptography dependencies. This technically alters the user-facing API,
  but not in way that should affect anyone since it only added a few extra trait
  contraints.

## 1.0.3 (2021-05-27)

#### Changed

- Allow `hmac-0.11`.

## 1.0.2 (2020-11-19)

#### Changed

- Allow `hmac-0.10`.

## 1.0.1 (2020-08-21)

#### Fixed

- A broken link in docstrings.

## 1.0.0 (2020-08-20)

Initial release.
