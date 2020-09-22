
#![allow(unused)]

use std::fmt;
use std::error::Error;

pub fn logic_err(e: &dyn Error) -> HTTPError {
    let s = format!("{}", e);
    HTTPError::LogicError(s)
}

#[derive(Debug)]
pub enum HTTPError {
    EncodingError(String),
    HttpError(http::Error),
    HyperError(hyper::Error),
    LogicError(String),
    StatusError(hyper::StatusCode),
    StorageError(String),
}

impl fmt::Display for HTTPError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let e: String;
        match self {
            HTTPError::EncodingError(he) => e = he.clone(),
            HTTPError::HttpError(he) => e = format!("{}", he),
            HTTPError::HyperError(he) => e = format!("{}", he),
            HTTPError::LogicError(s) => e = s.clone(),
            HTTPError::StorageError(s) => e = s.clone(),
            HTTPError::StatusError(sc) => e = format!("{}", sc),
        }
        write!(f, "HTTPError({})", e)?;
        Ok(())
    }
}

impl Error for HTTPError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            &HTTPError::HyperError(ref e) => Some(e),
            &HTTPError::HttpError(ref e) => Some(e),
            _ => None,
        }
    }
}

