//
//
//
use std::any::type_name;
use std::fs;
use validate_pgp_server::validate_certificate::config::Config;



/// setup a config object from source file
///
pub fn set_config() -> Config {
    let conf_file = "etc/validate_pgp_server.json";
    let cfgstring = fs::read_to_string(conf_file)
                      .expect("Unable to read config file");
    let cfg: Config = serde_json::from_str(&cfgstring).unwrap();
    return cfg;
}


/// not dead but being reported as such
#[allow(dead_code)]
pub fn get_type_of<T>(_: &T) -> String {
    format!("{}", type_name::<T>())
}


/// not dead but being reported as such
#[allow(dead_code)]
pub static SERVER_FUTURE_TYPE: &str =
     "validate_pgp_server::validate_certificate::sslserver::Server::server_run::{{closure}}";


/// not dead but being reported as such
#[allow(dead_code)]
pub static SERVER_TYPE: &str = "validate_pgp_server::validate_certificate::sslserver::Server";



/// default test config file location
#[allow(dead_code)]
pub static TEST_CONFIG_FILE: &str = "etc/validate_pgp_server.json";



/// macro to assist with async tests
/// 
#[macro_export]
macro_rules! async_wait_test_block {
     ($e:expr) => {
         tokio_test::block_on($e)
     };
}

//
//
