macro_rules! todo {
    ($info:expr) => {
        panic!("TODO `{}` at {}:{}", $info, file!(), line!())
    };
}

macro_rules! todo_field {
    ($value:expr) => {
        if !$value.is_empty() {
            todo!(stringify!($value));
        }
    };
}
