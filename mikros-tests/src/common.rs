
#[allow(dead_code)]
pub fn assets_path() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.push("resources/test");
    p
}
