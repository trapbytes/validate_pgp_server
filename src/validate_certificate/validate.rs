//
//
//
use bufstream::BufStream;
use chrono::Utc;
//
use openssl::ssl::{SslStream};
//
use openpgp::crypto::SessionKey;
use openpgp::types::SymmetricAlgorithm;
use openpgp::Result as pgpResult;
use openpgp::{Cert, packet::{PKESK, SKESK}};
use openpgp::parse::{Parse, stream::*};
//
use sequoia_openpgp as openpgp;
use sequoia_openpgp::policy::StandardPolicy;
use serde_json::Value;
// 
use std::net::{TcpStream};
use std::io::{Read, Write, Error};
use std::fmt;
//
//
use crate::validate_certificate::config::*;


// Start tokens from pgp encrypted data
pub static START_OF_PGP_MSG: &str = "-----BEGIN PGP MESSAGE-----";

// End tokens from pgp encrypted data
pub static END_OF_PGP_MSG: &str = "-----END PGP MESSAGE-----";

// Client default header msg
pub static CLIENT_HEADER: &str = "PGP_CHK_REQ";


// keys in the decrypted data - which in our case is json data
pub static JS_ORG: &str = "/org";
pub static JS_ORG_KEY: &str = "/license/org_key";
pub static JS_CLIENT_KEY: &str = "/license/client_key";

// max capture buffer size - used to collect the client data
pub const MAX_BUF_SIZE: usize = 4096;




/// hold the password used to decode the encrypted data
///
struct Helper { 
    config_password: String,
}


/// impl VerificationHelper used to decode the encrypted data
///
impl VerificationHelper for Helper {
    fn get_certs(&mut self, _ids: &[openpgp::KeyHandle]) -> pgpResult<Vec<Cert>> {
        Ok(Vec::new()) // Feed the Certs to the verifier here...
    }
    fn check(&mut self, _structure: MessageStructure) -> pgpResult<()> {
        Ok(()) // Implement your verification policy here.
    }
}


/// decryption helper implementation to decrypt the client data
///
impl DecryptionHelper for Helper {
    fn decrypt<D>(&mut self, _: &[PKESK], skesks: &[SKESK],
                  _sym_algo: Option<SymmetricAlgorithm>,
                  mut decrypt: D) -> pgpResult<Option<openpgp::Fingerprint>>
        where D: FnMut(SymmetricAlgorithm, &SessionKey) -> bool
    {
        // decrypt message
        let _x = skesks[0].decrypt(&self.config_password.clone().into())
                      .map(|(algo, session_key)| decrypt(algo, &session_key));
        Ok(None)
    }
}


///
///
pub trait ValidationHelperRoutines {

    /// exit function
    ///
    fn exit(&mut self, msg: &String) -> ();

    /// validate the client header
    ///
    fn validate_client_header(&mut self, msg: &String) -> bool;

    /// validate the client stream
    ///
    fn validate_stream_data(&mut self, msg: &String) -> bool;

    /// validate the client decrypted stream
    ///
    fn validate_decrypted_data(&mut self, msg: Vec<u8>) -> bool;

    /// send connected client a fail message
    ///
    fn send_client_fail(&mut self) -> ();

    /// send connected client a success message
    ///
    fn send_client_ok(&mut self, data: &[u8] ) -> bool;

    /// decrypt the client stream data
    ///
    fn decrypt_stream(&self, key: String, cert_data: String) -> Result<Vec<u8>,anyhow::Error>
    {
       let p = &StandardPolicy::new();
       let h = Helper { config_password : key };
       // decrypt the data from the client
       let mut v = DecryptorBuilder::from_bytes(&cert_data[..])?
                                    .with_policy(p, None, h)?;
       //
       let mut content = Vec::new();
       v.read_to_end(&mut content)?;
      
       Ok(content)
    }

    /// parse the incoming client message
    ///
    fn parse_client_msg(&mut self) -> Result<(String,String),Error>;
}


/// Assists with encapsulating data sent from the client
///
pub struct ValidationHelper<'a> {
    client_header: &'a str,
    start_of_pgp_msg: &'a str,
    end_of_pgp_msg: &'a str,
    peer_address: &'a str,
    client_stream: BufStream<&'a mut SslStream<TcpStream>>,
}


/// validation helper Display implementation
///
impl fmt::Display for ValidationHelper<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
           write!(f, "({})", self.client_header)
    }
}


/// Assists with encapsulating data sent from the client
///
impl <'a>ValidationHelper<'a> {

    /// create a new ValidationHelper
    ///
    pub fn new(cs: &'a mut SslStream<TcpStream>, peer_addr: &'a str, chdr: &'a str, spgp: &'a str, epgp: &'a str) 
        -> Result<ValidationHelper<'a>, Error>
    {  
       let buf_stream = BufStream::new(cs);
       Ok(ValidationHelper {
                            client_header: chdr,
                            start_of_pgp_msg: spgp,
                            end_of_pgp_msg: epgp,
                            peer_address: peer_addr,
                            client_stream: buf_stream
                           })
    }
}


/// basic implementaion of helper routines
///
impl ValidationHelperRoutines for ValidationHelper<'_> {

    /// exit function
    ///
    fn exit(&mut self, _msg: &String) -> () {
       ()
    }

    /// validate stream data from client
    ///
    fn validate_stream_data(&mut self, data: &String) -> bool { 
       if (!data.contains(self.start_of_pgp_msg) ||
           !data.contains(self.end_of_pgp_msg)) == true {       // invalid valid pgp message
           return false
       }
       return true
    }

    /// validate a client header
    ///
    fn validate_client_header(&mut self, data: &String) -> bool {
       if !data.contains(self.client_header) {                // invalid client header
          return  false 
       }
       return true
    }

    /// send the client a failure message
    ///
    fn send_client_fail(&mut self ) -> () {
       let _x = self.client_stream.write("unknown request\r\n\r\n".as_bytes());
       ()
    }

    /// send the client a success message
    ///
    fn send_client_ok(&mut self, data: &[u8] ) -> bool {
       let _x = self.client_stream.write(data);
       true
    }

    /// parse the client message
    ///
    fn parse_client_msg(&mut self) -> Result<(String,String),Error> {
       let mut client_buff = [0; MAX_BUF_SIZE];
       let read_len = self.client_stream.read(&mut client_buff).unwrap();
       
       let msg = String::from_utf8_lossy(&client_buff[0..read_len]);
       let mut msg_iter = msg.split(';');
       let hdr = msg_iter.next().unwrap();
       let mut ldata = msg_iter.next().unwrap();
       ldata = ldata.trim_end_matches("\r\n\r\n");

       Ok((hdr.to_string(), ldata.to_string()))
    }

    /// validate the client data sent to us
    ///
    fn validate_decrypted_data(&mut self, msg: Vec<u8>) -> bool {
       // our validate is based on extracting json data from the data
       let x: Value = match serde_json::from_slice(&msg) {
                         Ok(val) => { val },
                         Err(e) => { 
                             loginfo(format!("{} Error: json transform '{}'", self.peer_address, e));
                             Value::Null
                         }
                      };
       //
       if x == Value::Null {
          return false;
       }
       let bits: usize = 0;
       //
       let (_org, bits) = extract_json_parts(&x, JS_ORG, bits).unwrap();
       let (_org_key, bits) = extract_json_parts(&x, JS_ORG_KEY, bits).unwrap();
       let (_client_key, bits) = extract_json_parts(&x, JS_CLIENT_KEY, bits).unwrap();

       if bits == 3 {   // Success
          let _x = self.send_client_ok(x.to_string().as_bytes());
          return true
       }
       let _x = self.send_client_fail();
       false
  }

}


/// utility debug function
///
pub fn debuginfo(conf: &Config, msg: String) {
    if conf.debug == true {
       println!("{} - {}", Utc::now().to_rfc2822(), msg);
    }
}


/// utility logging function
///
pub fn loginfo(msg: String) {
    println!("{} - {}", Utc::now().to_rfc2822(), msg);
}


/// utility function to extract information from the decrytped data - which is json in our case
///
fn extract_json_parts(json: &Value, item: &str, bits: usize ) -> Result<(String,usize), Error> {
   match json.pointer( item ) {
      Some(a) => { Ok((a.to_string(), bits+1)) },
      None => { Ok(("none".to_string(), 0)) }
   }
}
