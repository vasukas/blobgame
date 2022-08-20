pub fn ron_load<T: serde::de::DeserializeOwned>(filename: &str) -> Option<T> {
    ron::from_str(&std::fs::read_to_string(filename).ok()?).ok()
}

pub fn ron_save<T: serde::Serialize>(filename: &str, value: &T) {
    std::fs::write(
        filename,
        ron::ser::to_string_pretty(value, Default::default()).unwrap(),
    )
    .unwrap()
}
