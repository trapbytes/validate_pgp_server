//
//
//
use easy_args::{ArgType, arg_spec};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Error, ErrorKind};
//
//

use crate::validate_certificate::ValidateConfigResult;



/// configuration for the tool
///
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub listen_address: String,
    pub cert_file: String,
    pub key_file: String,
    pub allowed_cert_thumbprints: Vec<String>,
    pub host_by_thumbprint: bool,
    pub host_by_cert: bool,
    pub auth_ssl: bool,
    pub active: bool,
    pub runtime_worker_threads: usize,
    pub runtime_worker_blocking_threads: usize,
    pub auth_key: Vec<String>,
    pub debug: bool,
}


/// helper for configuration object
///
pub trait ConfigHelper {
    fn validate_report_config(&mut self) -> &Config;
    fn is_config_valid(&self) -> bool;
}


/// implement configuration for the tool
///
impl Config {

   /// Look only for config file at specified path
   ///
   pub fn from_config_file(path: &str) -> ValidateConfigResult
   {
       let cfg_string = fs::read_to_string(path).expect("Unable to read config file");
       let cfg: Config = match serde_json::from_str(&cfg_string) {
                            Ok(c) => { c },
                            Err(e) => { return Err(Error::new(ErrorKind::Other, format!("{:?}",e))); }
                         };
       Ok(cfg)
   }

   /// read config file from a string path
   ///
   pub async fn from_config_file_async(path: &str) -> ValidateConfigResult {
       return Config::from_config_file(path)
   }

   /// Look only for --config-file argument and --help|? argument
   ///
   ///
   /// Panics:
   ///  panic on bad command line arguements
   ///
   pub fn from_command_line() -> ValidateConfigResult
   {
       let argspec = arg_spec!(config_file: String, help: String);
       let args = argspec.parse().unwrap();
       //
       if argspec.has_arg("config_file", ArgType::String) {
          if let Some(conf_file) = args.string("config_file")
          {
             return Config::from_config_file( conf_file );
          } 
          else if let Some(_helparg) = args.string("help") 
          {
             println!("Help: validate_pgp_server --config_file <path> --help");
             std::process::exit(0);
          } 
          return Err(Error::new(ErrorKind::Other, "no --config_file argument supplied"));
       }
       return Err(Error::new(ErrorKind::Other, "no --config_file argument supplied"));
   }
}


/// implement Config helper functions
///
impl ConfigHelper for Config {

   fn is_config_valid(&self) -> bool {
       if self.active {
          return true;
       }
       false
   }

   fn validate_report_config(&mut self) -> &Config {
       self.active = true;
       self
   }
}

/// implement Partial eq for Config
///
impl PartialEq for Config {
     fn eq(&self, other:&Self) -> bool {
           self.listen_address == other.listen_address
     }
}

/// implement Eq for Config
///
impl Eq for Config {}

//
//
