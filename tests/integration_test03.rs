//
//
//
//

mod common;



#[cfg(test)]
mod test_future {

    use super::*;
    use validate_pgp_server::config::{ConfigHelper};
    use validate_pgp_server::sslserver::Server;
    use crate::common::get_type_of;

    #[tokio::test]
    async fn server_check_returned_future(){
       let mut cfg = common::set_config();
       cfg.validate_report_config();
       let server = Server::new(&cfg).unwrap();
       let server_run = server.server_run(cfg);
       let type_of = get_type_of(&server_run);
       assert_eq!(common::SERVER_FUTURE_TYPE, type_of);
    }
}
