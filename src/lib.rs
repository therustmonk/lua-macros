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
    ($state:ident, $($from:ty),+) => {{
        use $crate::lua::Index;
        let names = [$(stringify!($from),)+];
        let quantity = names.len() as Index;
        auto_cleanup!($state, {
            let mut collect = || {
                let top = $state.get_top() - quantity;
                if top < 0 {
                    return Err(quantity + top + 1);
                }
                let mut position = 0;
                let result = ($({
                    position += 1;
                    let opt = $state.to_type::<$from>(top + position);
                    match opt {
                        Some(v) => v,
                        None => {
                            return Err(position);
                        },
                    }
                },)+);
                Ok(result)
            };
            collect()
        })
    }};
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
                    if let Ok((name, value)) = convert_arguments!(state, $key, $val) {
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
