//! This crate contains useful macros for `lua` crate.
//!
//! ## Clean up the stack for scope
//!
//! Use `auto_cleanup` to revert top of stack to size before scope:
//!
//! ```rust
//! # #[macro_use]
//! # extern crate lua_macros;
//! # use lua_macros::lua::State;
//!
//! fn main() {
//!     let mut state = State::new();
//!
//!     state.push(1);
//!
//!     auto_cleanup!(state, {
//!         state.push(2);
//!         state.push(3);
//!         state.push(4);
//!         state.push(5);
//!     });
//!
//!     assert_eq!(state.get_top(), 1);
//! }
//! ```
//!
//! ## Convert arguuments from lua
//!
//! Library has macro `convert_arguments` to convert arguments for
//! any types which implemented `FromLua` trait:
//!
//! ```rust
//! # #[macro_use]
//! # extern crate lua_macros;
//! # use lua_macros::lua::{State, Integer, Number};
//! # use lua_macros::lua::ffi::lua_State;
//! # use lua_macros::lua::libc::c_int;
//!
//! pub unsafe extern "C" fn fun_function(ls: *mut lua_State) -> c_int {
//!     let mut state = State::from_ptr(ls);
//!     let (_int, _float, _, _str) = convert_arguments!(state, Integer, Number, _, String)
//!         .map_err(|pos| {
//!             let msg = match pos {
//!                 1 => "integer | integer expected as first argument",
//!                 2 => "number | float expected as second argument",
//!                 3 => "any | third argument expected",
//!                 4 => "string | string expected as fourth argument",
//!                 _ => "unknown argument for `do` funciton",
//!             };
//!             state.arg_error(pos, msg);
//!         }).unwrap();
//!     state.push_string("That's OK!");
//!     1
//! }
//!
//! fn main() {
//!     let mut state = State::new();
//!     state.push_fn(Some(fun_function));
//!     state.set_global("fun");
//!
//!     assert!(state.do_string("return fun()").is_err());
//!     assert!(state.to_str(-1).unwrap().contains("bad argument #1"));
//!
//!     assert!(state.do_string("return fun(1)").is_err());
//!     assert!(state.to_str(-1).unwrap().contains("bad argument #2"));
//!
//!     assert!(state.do_string("return fun(1, 2.3)").is_err());
//!     assert!(state.to_str(-1).unwrap().contains("bad argument #3"));
//!
//!     assert!(state.do_string("return fun(1, 2.3, function() end)").is_err());
//!     assert!(state.to_str(-1).unwrap().contains("bad argument #4"));
//!
//!     assert!(!state.do_string("return fun(1, 2.3, {}, \"string\")").is_err());
//!     assert!(state.to_str(-1).unwrap().contains("OK"));
//!
//!     assert!(state.do_string("return fun({}, 2.3, {}, \"string\")").is_err());
//!     assert!(state.to_str(-1).unwrap().contains("bad argument #1"));
//!
//!     assert!(state.do_string("return fun(1, 2.3, {}, \"string\", \"extra\")").is_err());
//!     assert!(state.to_str(-1).unwrap().contains("bad argument #5"));
//! }
//! ```
//!
//! ## Read `HashMap` from Lua's table
//!
//! Macro `lua_table_type` creates wrapper type to unpack tables:
//!
//! ```rust
//! # #[macro_use]
//! # extern crate lua_macros;
//! # use lua_macros::lua::{State, Integer};
//!
//! lua_table_type!(UserTable<String, Integer>);
//!
//! fn main() {
//!     let mut state = State::new();
//!     state.do_string("return {one = 1, two = 2, three = 3}");
//!     let UserTable(mut table) = state.to_type(-1).unwrap();
//!     assert_eq!(table.remove("one"), Some(1));
//!     assert_eq!(table.remove("two"), Some(2));
//!     assert_eq!(table.remove("three"), Some(3));
//! }
//! ```
//!
//! ## Read `Vec` from Lua's array
//!
//! Macro `lua_array_type` creates wrapper type to unpack arrays:
//!
//! ```rust
//! # #[macro_use]
//! # extern crate lua_macros;
//! # use lua_macros::lua::{State, Integer};
//!
//! lua_array_type!(UserArray<Integer>);
//!
//! fn main() {
//!     let mut state = State::new();
//!     state.do_string("return {1, 2, 3}");
//!     let UserArray(array) = state.to_type(-1).unwrap();
//!     assert_eq!(array[0], 1);
//! }
//! ```
//!
//! ## Adds own userdata
//!
//! Macro `lua_userdata` implements userdata features for type:
//!
//! ```rust
//! # #[macro_use]
//! # extern crate lua_macros;
//! # use lua_macros::lua::{State, Integer};
//!
//! # #[derive(Clone, Debug, PartialEq)]
//! enum UserEnum {
//!   One,
//!   Two,
//!   Three,
//! }
//!
//! lua_userdata!(UserEnum);
//!
//! fn main() {
//!     let mut state = State::new();
//!     UserEnum::attach(&mut state);
//!
//!     let ud = UserEnum::One;
//!     state.push(ud);
//!     state.push_nil();
//!
//!     let restored = state.to_type::<UserEnum>(-2).unwrap();
//!     let wrong = state.to_type::<UserEnum>(-1);
//!
//!     assert_eq!(restored, UserEnum::One);
//!     assert!(wrong.is_none());
//! }
//! ```


pub extern crate lua;

/// Clean up stack for the scope.
#[macro_export]
macro_rules! auto_cleanup {
    ($state:ident, $b:block) => {{
        let top = $state.get_top();
        let result = $b;
        $state.set_top(top);
        result
    }};
}

/// Convert arguments using `FromLua` trait.
#[macro_export]
macro_rules! convert_arguments {
    (@strict $strict:expr, $state:ident, $($from:tt),+) => {{
        use $crate::lua::Index;
        let names = [$(stringify!($from),)+];
        let quantity = names.len() as Index;
        let top = $state.get_top();
        auto_cleanup!($state, {
            let mut collect = || {
                let base = $state.get_top() - quantity;
                if base < 0 {
                    return Err(top + 1); // +1 because next arg expected
                }
                if $strict && base > 0 {
                    return Err(top);
                }
                let mut position = 0;
                let result = ($({
                    position += 1;
                    convert_arguments!(@unpack $from, $state, base, position)
                },)+);
                Ok(result)
            };
            collect()
        })
    }};
    (@unpack _, $state:ident, $base:expr, $position:expr) => {()};
    (@unpack $from:ty, $state:ident, $base:expr, $position:expr) => {{
        let opt = $state.to_type::<$from>($base + $position);
        match opt {
            Some(v) => v,
            None => {
                return Err($position);
            },
        }
    }};
    ($state:ident, $($from:tt),+) =>
        (convert_arguments!(@strict true, $state, $($from),+));
}

/// Makes wrapper to read table to hash map.
///
/// This macro add wrapper struct, because impossible to implement `FromLua` to `HashMap`
/// because they are from other crates.
#[macro_export]
macro_rules! lua_table_type {
    ($name:ident < $key:ty , $val:ty >) => {
        pub struct $name(pub ::std::collections::HashMap<$key, $val>);

        impl $crate::lua::FromLua for $name {
            fn from_lua(state: &mut $crate::lua::State, index: $crate::lua::Index) -> Option<Self> {
                if !state.is_table(index) {
                    return None;
                }
                let mut map = ::std::collections::HashMap::new();
                let index = state.abs_index(index);
                state.push_nil();
                while state.next(index) {
                    // Non-strict, because this macro pushes to stack additional values
                    if let Ok((name, value)) = convert_arguments!(@strict false, state, $key, $val) {
                        map.insert(name, value);
                        state.pop(1); // Pop `key` only
                    } else {
                        state.pop(2); // Pop `key` and `value`, because `next` call returned `true`
                        return None;
                    }
                }
                Some($name(map))
            }
        }
    };
}

/// Makes wrapper to read table to array.
#[macro_export]
macro_rules! lua_array_type {
    ($name:ident < $val:ty >) => {
        pub struct $name(pub ::std::vec::Vec<$val>);

        impl $crate::lua::FromLua for $name {
            fn from_lua(state: &mut $crate::lua::State, index: $crate::lua::Index) -> Option<Self> {
                if !state.is_table(index) {
                    return None;
                }
                let mut vec = ::std::vec::Vec::new();
                let index = state.abs_index(index);
                for idx in 1.. {
                    state.geti(index, idx);
                    if state.is_nil(-1) {
                        state.pop(1);
                        break;
                    }
                    if let Ok((value,)) = convert_arguments!(@strict false, state, $val) {
                        vec.push(value);
                        state.pop(1);
                    } else {
                        state.pop(1);
                        return None;
                    }
                }
                Some($name(vec))
            }
        }

        impl $crate::lua::ToLua for $name {
            fn to_lua(&self, state: &mut $crate::lua::State) {
                let $name(ref vec) = *self;
                state.new_table();
                let mut idx = 0;
                for item in vec {
                    idx += 1; // Starts from 1 too
                    state.push(item.to_owned());
                    state.raw_seti(-2, idx);
                }
            }
        }
    };
}

/// Add userdata's methods to user's type.
#[macro_export]
macro_rules! lua_userdata {
    ($ud:ident $(, $field:expr => $func:ident )*) => {
        impl $ud {
            pub fn meta_name() -> &'static str {
                concat!(stringify!($ud), ".Rust")
            }

            pub fn attach(state: &mut State) {
                let created = state.new_metatable($ud::meta_name());
                $(
                state.push_fn(Some($func));
                state.set_field(-2, $field);
                )*
                state.pop(1); // pop metatable
                if !created {
                    panic!("Metatable '{}' already exists.", $ud::meta_name());
                }
            }
        }

        impl $crate::lua::FromLua for $ud {
            fn from_lua(state: &mut $crate::lua::State, index: $crate::lua::Index) -> Option<Self> {
                unsafe {
                    state.test_userdata_typed::<$ud>(index, $ud::meta_name())
                        .map(|p| p.clone())
                }
            }
        }

        impl $crate::lua::ToLua for $ud {
            fn to_lua(&self, state: &mut $crate::lua::State) {
                unsafe { *state.new_userdata_typed() = self.clone(); }
                state.set_metatable_from_registry($ud::meta_name());
            }
        }
    };
}
