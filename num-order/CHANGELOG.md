# Changelog

## [1.2.0] - 2023-08-30

- Bump the version of `num-modular` to prevent a bug.
- The hash values of negative infinity and NaN are changed.
- The hash values of rationals with a multiple of M127 are changed.

> It's not released as a new major version because the previously designed values are not intended. It's set by mistake.

## [1.1.0] - 2023-08-29

Now the crate `num-traits` is an optional dependency, and the `libm` dependency is removed. The dependency version of `num-modular` is updated to v0.6.

## [1.0.4] - 2022-05-23

Bump the version of dependency `num-modular`.

## [1.0.3] - 2022-04-17

Bump the version of dependency `num-modular`.

## [1.0.2] - 2022-04-06

Bump the version of dependency `num-modular`, use `MersenneInt` for more efficient hashing.

## [1.0.1] - 2022-03-31

First public stable version of `num-order`! Numerical consistant order and hash comparison are fully supported for following types:
- Unsigned integers: `u8`, `u16`, `u32`, `u64`, `u128`
- Signed integers: `i8`, `i16`, `i32`, `i64`, `i128`
- Float numbers: `f32`, `f64`
- (`num-rational`) Rational numbers: `Ratio<i8>`, `Ratio<i16>`, `Ratio<i32>`, `Ratio<i64>`, `Ratio<i128>`
- (`num-complex`) Complex numbers: `Complex<f32>`, `Complex<f64>`

> v1.0.0 was yanked because `num-rational` is accidentally added as default feature.
