use std::error::Error;
pub type MyResult<T> = Result<T, Box<dyn Error>>;
