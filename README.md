# const-assoc

A `no_std`-compatible, const-capable associative array with minimal or no runtime overhead.

Currently, keys are limited to enums with a primitive representation. In the future,
it might be possible to support other types, possibly at the expense of not exposing
`const`-qualified methods for these key types or some runtime overhead.

# Example
 ```rust
use const_assoc::{assoc, PrimitiveEnum};

#[repr(u8)]
#[derive(Copy, Clone, PrimitiveEnum)]
enum Letter {
    A,
    B,
    C,
}

fn main() {
    let letters = assoc! {
        Letter::A => 'a',
        Letter::B => 'b',
        Letter::C => 'c',
    };

    assert_eq!(letters[Letter::A], 'a');
    assert_eq!(letters[Letter::C], 'c');
}
```