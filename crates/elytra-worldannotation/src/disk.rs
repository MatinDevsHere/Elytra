pub enum EwaIoError {
    IoError(tokio::io::Error),
    ParseError,
}

pub fn save_ewa(world: &str, annotation: &str) -> Result<(), EwaIoError> {
    panic!("Not implemented");
}

pub fn load_ewa(world: &str) -> Result<String, EwaIoError> {
    panic!("Not implemented");
}
