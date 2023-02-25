use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorBody {
    pub error_message: String,
}

impl fmt::Display for ErrorBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ApiError {
    EmptyFlightPaths(ErrorBody),
    InvalidFlightPath(ErrorBody),
    NoStartingAirportDiscovered(ErrorBody),
    NoEndingAirportDiscovered(ErrorBody),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ApiError {}
