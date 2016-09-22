pub extern crate lua;

#[macro_export]
macro_rules! auto_cleanup {
    ($state:ident, $b:block) => {{
        let top = $state.get_top();
        let result = $b;
        $state.set_top(top);
        result
    }};
}

/// ```rust
/// convert_arguments!(state, Number, String);
/// ```
#[macro_export]
macro_rules! convert_arguments {
    (STRICT $strict:expr, $state:ident, $($from:tt),+) => {{
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
                    convert_arguments!(UNPACK $from, $state, base, position)
                },)+);
                Ok(result)
            };
            collect()
        })
    }};
    (UNPACK _, $state:ident, $base:expr, $position:expr) => {()};
    (UNPACK $from:ty, $state:ident, $base:expr, $position:expr) => {{
        let opt = $state.to_type::<$from>($base + $position);
        match opt {
            Some(v) => v,
            None => {
                return Err($position);
            },
        }
    }};
    ($state:ident, $($from:tt),+) =>
        (convert_arguments!(STRICT true, $state, $($from),+));
}

#[macro_export]
macro_rules! lua_map_table {
    ($name:ident < $key:ty , $val:ty >) => {
        struct $name(HashMap<$key, $val>);

        impl FromLua for $name {
            fn from_lua(state: &mut State, index: Index) -> Option<Self> {
                let mut map = HashMap::new();
                let index = state.abs_index(index);
                state.push_nil();
                while state.next(index) {
                    // Non-strict, because this macro pushes to stack additional values
                    if let Ok((name, value)) = convert_arguments!(STRICT false, state, $key, $val) {
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

#[macro_export]
macro_rules! lua_userdata {
    ($ud:ident $(, $field:expr => $func:ident )*) => {
        impl $ud {
            fn meta_name() -> &'static str {
                concat!(stringify!($ud), ".Rust")
            }

            fn attach(state: &mut State) {
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

        impl FromLua for $ud {
            fn from_lua(state: &mut State, index: Index) -> Option<Self> {
                unsafe {
                    state.test_userdata_typed::<$ud>(index, $ud::meta_name())
                        .map(|p| p.clone())
                }
            }
        }

        impl ToLua for $ud {
            fn to_lua(&self, state: &mut State) {
                unsafe { *state.new_userdata_typed() = self.clone(); }
                state.set_metatable_from_registry($ud::meta_name());
            }
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
