/*
#[macro_export]
macro_rules! unpack_field {
    ($state:ident, Index ( $idx:expr )) => {
        $state.geti(-1, $idx);
    };
    ($state:ident, Field ( $idx:expr )) => {
        $state.get_field(-1, $idx);
    };
    ($state:ident, $pos:expr, Index ( $idx:expr )) => {
        $state.geti($pos, $idx);
    };
    ($state:ident, $pos:expr, Field ( $idx:expr )) => {
        $state.get_field($pos, $idx);
    };
}
*/

#[macro_export]
macro_rules! auto_cleanup {
    ($state:ident, $b:block) => {{
        let top = $state.get_top();
        let result = $b;
        let new_top = $state.get_top();
        $state.pop(new_top - top);
        result
    }};
}

/*
#[macro_export]
macro_rules! ipairs {
    ($state:ident, $b:block) => {{
        ensure_table!($state);
        let mut idx = 0;
        loop {
            idx += 1;
            $state.geti(-1, idx);
            if $state.is_nil(-1) {
                $state.pop(1);
                break;
            } else {
                auto_cleanup!($state, $b);
                $state.pop(1);
            }
        }
    }};
}
*/

/*
/// ```rust
/// ensure_types!(state, is_number, is_bool, is_fn);
/// ```
#[macro_export]
macro_rules! ensure_types {
    ($state:ident, $($check:ident),+) => {
        let names = [$(stringify!($check),)+];
    };
}

#[macro_export]
macro_rules! ensure_table {
    ($state:ident) => {
        if !$state.is_table(-1) {
            $state.arg_error(1, "Table expected");
        }
    };
}
*/

/*
/// ```rust
/// convert_table!(state, Index(1): String, Field("value"): Number);
/// ```
#[macro_export]
macro_rules! convert_table {
    ($state:ident, $($tp:ident ( $idx:expr ) : $from:ty),+) => {{
        // TODO Insert auto_cleanup
        auto_cleanup!($state, {
            let top = $state.get_top();
            ensure_table!($state);
            $({
                unpack_field!($state, top, $tp ( $idx ));
                if $state.is_none_or_nil(-1) {
                    let msg = format!("Not valid or nil value by index '{}'", $idx);
                    $state.arg_error(1, &msg);
                }
            })+
            let result = convert_arguments!($state, $($from),+);
            result
        })
    }};
}
*/

/// ```rust
/// convert_arguments!(state, Number, String);
/// ```
#[macro_export]
macro_rules! convert_arguments {
    ($state:ident, $($from:ty),+) => {{
        let names = [$(stringify!($from),)+];
        let quantity = names.len() as Index;
        auto_cleanup!($state, {
            let mut collect = || {
                let top = $state.get_top() - quantity;
                if top < 0 {
                    return Err(quantity + top);
                }
                let mut position = 0;
                let result = ($({
                    position += 1;
                    let opt = $state.to_type(top + position).map(|v: $from| v);
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
