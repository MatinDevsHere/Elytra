pub enum EwaIoError {
    IoError(tokio::io::Error),
    ParseError,
}

pub fn save_ewa(_world: &str, _annotation: &str) -> Result<(), EwaIoError> {
    panic!("Not implemented");
}

pub fn load_ewa(_world: &str) -> Result<String, EwaIoError> {
    panic!("Not implemented");
}
