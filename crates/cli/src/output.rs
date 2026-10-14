use serde::Serialize;

pub fn print_json<T: Serialize + ?Sized>(value: &T) {
    println!("{}", serde_json::to_string_pretty(value).expect("value serializes to JSON"));
}
