//
//
//
//
use async_trait::async_trait;
use chrono::Utc;
use openssl::ssl::{SslMethod, SslAcceptor, SslFiletype, SslStream};
//
use std::collections::{HashSet};
use std::io::{Write, Error, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use tokio::runtime::Builder;


use crate::validate_certificate::config::*;
use crate::validate_certificate::validate::*;





///
///
pub struct Server {
   pub tls_config: Arc<openssl::ssl::SslAcceptor>,
   pub tcp_listener: TcpListener,
   pub allowed_certs_thumbprints: HashSet<String>,
   pub host_by_cert: bool,         // change to validate_by_cer
   pub host_by_thumbprint: bool,   // change to validate_by_thumbprint
}


/// server helper used to override server methods
///
#[async_trait]
pub trait ServerHelper {
    fn new(config: &Config) -> Result<Server, Error>;
    fn authenticate_certificate(&self, stream: &mut SslStream<TcpStream>) -> Result<bool, Error>;
    async fn server_run(&self, conf: Config);
    async fn process_messages(&self, conf: Config) -> Result<(), Error>;
}


///
///
impl Server {

   /// new Server 
   ///
   pub fn new(config: &Config) -> Result<Server, Error> {
       let mut allowed = HashSet::new();
       // collect the allowed thumbprints into a hash set
       config.allowed_cert_thumbprints
             .iter()
             .for_each(|t| { allowed.insert(t.to_lowercase()); });
       //
       let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
       acceptor.set_private_key_file(&config.key_file, SslFiletype::PEM).unwrap();
       acceptor.set_certificate_chain_file(&config.cert_file).unwrap();
      
       // accept all certificates, we'll do our own validation on them
       acceptor.set_verify_callback(openssl::ssl::SslVerifyMode::PEER, |_, _| true);
       let acceptor = Arc::new(acceptor.build());
       let listener = match TcpListener::bind(&config.listen_address) {
                         Ok(l) => { l },
                         Err(e) => {
                             return Err(Error::new(ErrorKind::Other, format!("Server startup Error: {:?}", e)))
                         }
                      };
       //
       Ok(Server {tls_config: acceptor, tcp_listener: listener, allowed_certs_thumbprints: allowed,
                  host_by_cert: config.host_by_cert, host_by_thumbprint: config.host_by_thumbprint})
   }


   /// our tls certificate authentication function
   ///
   pub fn authenticate_certificate(&self, stream: &mut SslStream<TcpStream>) -> Result<bool, Error> {
       fn get_friendly_name(peer: &openssl::x509::X509) -> String {
            peer.subject_name()
                .entries()
                .last()
                .map(|it| it.data()
                            .as_utf8()
                            .and_then(|s| Ok(s.to_string()))
                            .unwrap_or("".to_string())
                )
                .unwrap_or("<Unknown>".to_string())
       }
       match stream.ssl().peer_certificate() {
          None => {
               stream.write(b"ERR No certificate was provided\r\n")?;
               return Ok(false);
          }
          Some(peer) => {
               if self.host_by_cert {
                  let host_cert_ok = match peer.verify( &peer.public_key().unwrap()) {
                       Ok(msg) => { msg }
                       Err(emsg) => { 
                           return Err(Error::new(ErrorKind::Other, format!("cerificate peer-verify failed {:?}", emsg)));
                       }
                  };
                  return Ok(host_cert_ok);
               } else if self.host_by_thumbprint {
                  let thumbprint = hex::encode(peer.digest(openssl::hash::MessageDigest::sha1())?);
                  if self.allowed_certs_thumbprints.contains(&thumbprint) == false {
                      let msg = format!("ERR certificate ({}) thumbprint '{}' is unknown\r\n",
                                        get_friendly_name(&peer), thumbprint);
                      stream.write(msg.as_bytes())?;
                      return Ok(false);
                  }
               }
          }
       };
       return Ok(true);
   }

   /// start a server that accepts / decodes valid client connections
   ///
   pub async fn server_run(&self, conf: Config) {
       //
       let rt = Builder::new_multi_thread()
                    .enable_all()
                    .worker_threads(conf.runtime_worker_threads)
                    .max_blocking_threads(conf.runtime_worker_blocking_threads)
                    .build()
                    .unwrap();
       //
       for stream in self.tcp_listener.incoming() {
           match stream {
              Ok(stream) => {
                 let acceptor = self.tls_config.clone();
                 //
                 let mut client_stream;
                 match acceptor.accept(stream) {
                    Ok(cs) => { client_stream = cs; },
                    Err(e) => { 
                                loginfo(format!("{ } - stream accept error: '{:?}'",Utc::now().to_rfc2822(),e));
                                continue; // next iteration of for loop
                              }
                 };
                 let mut auth_ok = true;
                 if conf.auth_ssl == true {
                      if self.authenticate_certificate(&mut client_stream).unwrap() == false {
                         // Client certificate validation failed - close conn
                         client_stream.shutdown().unwrap();
                         auth_ok = false;
                      }
                 }
                 if auth_ok {
                    let n_conf = conf.clone();
                    rt.spawn(async move {
                               let _x = Server::process_messages(&mut client_stream, n_conf).await;
                               client_stream.shutdown().unwrap();
                            });
                 }
              },
              Err(e) => {     // connection failed
                 loginfo(format!("{} - Error connection failed: {:?}", Utc::now().to_rfc2822(),e));
              }
           }
       }
       //
       rt.shutdown_background();
   }

   /// process the client message and parse for the header and the encrypted data
   ///
   pub async fn process_messages(stream: &mut SslStream<TcpStream>, conf: Config) -> Result<(), Error> {
      //
      let peer_addr = match stream.get_ref().peer_addr() { 
                        Ok(r) => { r.to_string() },
                        Err(e) => { 
                            return Err(Error::new(ErrorKind::Other, format!("get_peer addr error '{:?}'",e))) 
                        }
                      };
      loginfo(format!("{} - {} New connection ok", Utc::now().to_rfc2822(), peer_addr));
      // New Parse client message via helper functions
      let mut vhelper =
              ValidationHelper::new(stream, &peer_addr, CLIENT_HEADER, START_OF_PGP_MSG, END_OF_PGP_MSG).unwrap();

      let (header, encrypted_data) = vhelper.parse_client_msg().unwrap();
      // validate the client header and data stream
      if !vhelper.validate_client_header(&header) || 
         !vhelper.validate_stream_data(&encrypted_data) 
      {
         vhelper.send_client_fail();
         return Ok(())
      }
      // decrypt the data
      match vhelper.decrypt_stream( conf.auth_key.join(""), encrypted_data ) {
         Ok(decrypted_data) => { 
            let _tf = vhelper.validate_decrypted_data(decrypted_data);
         },
         Err(e) => {   // Decrypt error
             vhelper.send_client_fail();
             return Err(Error::new(ErrorKind::Other,
                                   format!("{} - {} decrypt_error: '{:?}'", Utc::now().to_rfc2822(),peer_addr,e)));
         }
      };

      Ok(())
   }
}
