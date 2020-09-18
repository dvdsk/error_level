use error_level::ErrorLevel;
use log::Level;
use simplelog::{LevelFilter, Config, SimpleLogger};

#[allow(dead_code)]
#[derive(Debug, ErrorLevel)]
pub enum OuterError {
    #[report(info)]
    Error0,
    // Error1,
}


pub mod example_mod {
    use super::*;

    #[derive(Debug, ErrorLevel)]
    pub enum Error {
        #[report(info)]
        Error00,
    }
}

#[test]
fn simple() {
    #[derive(Debug, ErrorLevel)]
    pub enum CustomError {
        #[report(warn)]
        ErrorA,
        #[report(info)]
        ErrorB,
        #[report(no)]
        ErrorC,
        ErrorD(OuterError),
    }

    SimpleLogger::init(LevelFilter::Trace, Config::default()).unwrap();

    let a = CustomError::ErrorA;
    let b = CustomError::ErrorB;
    let c = CustomError::ErrorC;
    let d = CustomError::ErrorD(OuterError::Error0);

    assert_eq!(a.error_level(), Some(Level::Warn));
    assert_eq!(b.error_level(), Some(Level::Info));
    assert_eq!(c.error_level(), None);
    assert_eq!(d.error_level(), Some(Level::Info));
    b.log_error();
}

#[test]
fn with_path_seperator() {
    #[derive(Debug, ErrorLevel)]
    pub enum CustomError {
        #[report(warn)]
        ErrorA,
        #[report(info)]
        ErrorB,
        #[report(no)]
        ErrorC,
        ErrorD(example_mod::Error),
    }

    let a = CustomError::ErrorA;
    let b = CustomError::ErrorB;
    let c = CustomError::ErrorC;
    let d = CustomError::ErrorD(example_mod::Error::Error00);

    assert_eq!(a.error_level(), Some(Level::Warn));
    assert_eq!(b.error_level(), Some(Level::Info));
    assert_eq!(c.error_level(), None);
    assert_eq!(d.error_level(), Some(Level::Info));
    b.log_error();
}

#[test]
fn with_inner_attribute() {

    #[derive(thiserror::Error, Debug, ErrorLevel)]
    pub enum InnerE {
        #[report(no)]
        #[error("by")]
        Error,
    };

    const CONSTANT_STR: &str = "test string";
    #[derive(thiserror::Error, Debug, ErrorLevel)]
    pub enum CustomError {
        #[report(no)]
        #[error("hi, {}", CONSTANT_STR)]
        ErrorD(#[from] InnerE),
    }

    let d = CustomError::from(InnerE::Error);

    assert_eq!(d.error_level(), None);
}

