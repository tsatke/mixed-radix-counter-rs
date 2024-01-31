# Mixed Radix Counter

A counter for a mixed radix system.

The counter consists of `values` and `limits`.
<br/>
`values` overflow at their `limit` (exclusive).

Given `values` of `[0, 1, 4]` and `limits` of `[3, 4, 5]`, adding `1` to the counter results in the `values` `[0, 2, 0]`.

## Usage

```rust
let mut mrc = MixedRadixCounter::try_from_limits([u64::MAX, 365, 24, 60, 60]).expect("default values don't fit the limits");
assert_eq!(*mrc, [0, 0, 0, 0, 0]);

mrc.add(69_413_798); // or call `mrc.increment()` in a loop 69_413_798 times, but beware, that's a lot slower
assert_eq!(*mrc, [2, 73, 9, 36, 38]);
```
