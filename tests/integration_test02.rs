//
//
//
//

mod common;



#[cfg(test)]
mod async_tests {

    use super::*;
    use validate_pgp_server::config::{Config, ConfigHelper};
    use validate_pgp_server::sslserver::Server;

    #[tokio::test]
    async fn test_one() {
       let cfg = common::set_config();
       assert_eq!(
           Config::from_config_file_async(common::TEST_CONFIG_FILE).await.unwrap(),
           cfg
         );
    }


    #[test]
    fn config_check(){
        let cfg = Config::from_config_file(common::TEST_CONFIG_FILE).unwrap();
        assert_eq!(false, cfg.is_config_valid());
    }


    #[tokio::test]
    async fn server_check_03(){
       let mut cfg = common::set_config();
       cfg.validate_report_config();

       let server = Server::new(&cfg).unwrap();
       let type_of = common::get_type_of(&server);
       assert_eq!(common::SERVER_TYPE, type_of);
    }
}
