pub fn random_str(len: usize) -> String {
    uuid::Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(len)
        .collect()
}
