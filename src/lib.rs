pub trait ErrorLevel {
    fn error_level(&self) -> log::Level;
}
