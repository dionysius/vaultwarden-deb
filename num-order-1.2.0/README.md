Numerically consistent `Eq`, `Ord` and `Hash` implementations for various `num` types (`u32`, `f64`, `num_bigint::BigInt`, etc.).

# Example
```rust
use std::cmp::Ordering;
use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;
use num_order::{NumOrd, NumHash};

assert!(NumOrd::num_eq(&3u64, &3.0f32));
assert!(NumOrd::num_lt(&-4.7f64, &-4i8));
assert!(!NumOrd::num_ge(&-3i8, &1u16));

// 40_000_000 can be exactly represented in f32, 40_000_001 cannot
// 40_000_001 becames 40_000_000.0 in f32
assert_eq!(NumOrd::num_cmp(&40_000_000f32, &40_000_000u32), Ordering::Equal);
assert_ne!(NumOrd::num_cmp(&40_000_001f32, &40_000_001u32), Ordering::Equal);
assert_eq!(NumOrd::num_partial_cmp(&f32::NAN, &40_000_002u32), None);

// same hash values are guaranteed for equal numbers
let mut hasher1 = DefaultHasher::new();
3u64.num_hash(&mut hasher1);
let mut hasher2 = DefaultHasher::new();
3.0f32.num_hash(&mut hasher2);
assert_eq!(hasher1.finish(), hasher2.finish())
```
