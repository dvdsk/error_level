use error_level::ErrorLevel;

#[derive(Debug)]
enum ErrorWithoutImpl {
    Error0,
    Error1,
}

#[test]
fn does_not_implement_ErrorLevel() {
    #[derive(Debug, ErrorLevel)]
    pub enum CustomError {
        #[report(warn)]
        ErrorA,
        #[report(info)]
        ErrorB,
        #[report(no)]
        ErrorC,
        ErrorD(ErrorWithoutImpl),
    }

    let a = CustomError::ErrorA;
    let d = CustomError::ErrorD(ErrorWithoutImpl::Error1);
}

#[test]
fn missing_attributes() {
    #[derive(Debug, ErrorLevel)]
    pub enum CustomError {
        #[report(warn)]
        ErrorA,
        #[report(info)]
        ErrorB,
        ErrorC,
        ErrorD((String, String)),
    }
}
