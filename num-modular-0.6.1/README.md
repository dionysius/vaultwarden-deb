# num-modular

A generic implementation of integer division and modular arithmetics in Rust. It provide basic operators and an type to represent integers in a modulo ring. Specifically the following features are supported:

- Common modular arithmetics: `add`, `sub`, `mul`, `div`, `neg`, `double`, `square`, `inv`, `pow`
- Optimized modular arithmetics in **Montgomery form**
- Optimized modular arithmetics with **pseudo Mersenne primes** as moduli
- Fast **integer divisibility** check
- **Legendre**, **Jacobi** and **Kronecker** symbols

It also support various integer type backends, including primitive integers and `num-bigint`. Note that this crate also supports `[no_std]`. To enable `std` related functionalities, enable the `std` feature of the crate.

<!-- TODO: Roadmap for v1:
- maybe support invariant integer form?
- const functions (if const traits are stablized then)
-->
