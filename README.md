# Lua-macros library

This library provides useful macros to simplify embedding Lua to Rust.
It's based on [`lua`](https://github.com/jcmoyer/rust-lua53) crate.

Example:

```rust
#[macro_use]
extern crate lua_macros;
use lua_macros::lua::{State, Integer};

#[derive(Clone, Debug, PartialEq)]
enum UserEnum {
  One,
  Two,
  Three,
}

lua_userdata!(UserEnum);

fn main() {
    let mut state = State::new();
    UserEnum::attach(&mut state);

    let ud = UserEnum::One;
    state.push(ud);
    state.push_nil();

    let restored = state.to_type::<UserEnum>(-2).unwrap();
    let wrong = state.to_type::<UserEnum>(-1);

    assert_eq!(restored, UserEnum::One);
    assert!(wrong.is_none());
}
```

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
