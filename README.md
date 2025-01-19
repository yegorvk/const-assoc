# const_array_map

A `no_std`-compatible, const-capable Map type backed by an array.

This crate defines a new map type, `ConstArrayMap`, similar to other
data structures but implemented using a single array with
zero-cost conversion between keys and array indices.

Currently, keys are limited to enums with a primitive representation. In the future,
it might also be possible to support arbitrary types with a relatively small
number of distinct valid values, possibly at the expense of not exposing
`const`-qualified methods for these key types.

## Example

```rust
use const_array_map::{const_array_map, PrimitiveEnum};

#[repr(u8)]
#[derive(Copy, Clone, PrimitiveEnum)]
enum Letter {
    A,
    B,
    C,
}

fn main() {
    let letters = const_array_map! {
        Letter::A => 'a',
        Letter::B => 'b',
        Letter::C => 'c',
    };

    assert_eq!(letters[Letter::A], 'a');
    assert_eq!(letters[Letter::C], 'c');
}
```
