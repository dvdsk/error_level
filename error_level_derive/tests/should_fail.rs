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
        #[level(Warn)]
        ErrorA,
        #[level(Info)]
        ErrorB,
        #[level(No)]
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
        #[level(Warn)]
        ErrorA,
        #[level(Info)]
        ErrorB,
        ErrorC,
        ErrorD((String, String)),
    }
}
