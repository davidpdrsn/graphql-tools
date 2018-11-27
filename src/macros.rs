macro_rules! todo {
    ($info:expr) => {
        panic!("TODO `{}` at {}:{}", $info, file!(), line!())
    };
}

macro_rules! todo_field {
    ($value:ident, $field:ident) => {
        if !$value.$field.is_empty() {
            todo!(stringify!($value.$field));
        }
    };
}
