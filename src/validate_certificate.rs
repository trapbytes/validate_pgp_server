//
//
//


use std::io::Error;


pub mod config;
pub mod sslserver;
pub mod validate;


/// type to retrun if a proper config object was created
type ValidateConfigResult = Result<crate::config::Config, Error>;
