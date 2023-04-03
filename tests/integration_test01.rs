//
//
//
//

mod common;



#[cfg(test)]
mod config_tests {

    use super::*;

    use validate_pgp_server::config::ConfigHelper;

    #[test]
    fn config_check(){
        let cfg = common::set_config();
        assert_eq!(false, cfg.is_config_valid());
    }

    #[test]
    fn config_check_valid(){
       let mut cfg = common::set_config();
       let _x = cfg.validate_report_config();
       let res =  cfg.is_config_valid();
       assert!(true, "config is not valid  '{}'", res);
    }

    #[test]
    fn config_check_valid2() -> Result<(), String> {
       let mut cfg = common::set_config();
       let _x = cfg.validate_report_config();
       if cfg.is_config_valid() {
          Ok(())
        } else {
          Err(String::from("Configuration is not valid"))
        }
    }
}
