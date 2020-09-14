pub trait ErrorLevel : std::fmt::Debug {
    fn error_level(&self) -> Option<log::Level>;
    fn log_error(&self){
        match self.error_level() {
            None => (),
            Some(log::Level::Trace) => log::trace!("{:?}", &self),
            Some(log::Level::Debug) => log::debug!("{:?}", &self),
            Some(log::Level::Info) => log::info!("{:?}", &self),
            Some(log::Level::Warn) => log::warn!("{:?}", &self),
            Some(log::Level::Error) => log::error!("{:?}", &self),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use simplelog::{LevelFilter, Config, SimpleLogger};

    #[derive(Debug)]
    enum TestError {
        Error0,
        Error1,
    }

    impl ErrorLevel for TestError {
        fn error_level(&self) -> Option<log::Level> {
            match self {
                Self::Error0 => None,
                Self::Error1 => Some(log::Level::Warn),
            }
        }
    }

    #[test]
    fn test(){
        SimpleLogger::init(LevelFilter::Trace, Config::default()).unwrap();

        let e0 = TestError::Error0;
        let e1 = TestError::Error1;
        
        println!("this test should output something like: time [WARN] Error1");
        e0.log_error();
        e1.log_error();
    }
}
