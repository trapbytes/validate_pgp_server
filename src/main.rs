//
//
//
//
use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;
use std::fs;
use std::path::Path;
use std::{thread};
use std::time::Duration;
//
use validate_pgp_server::validate_certificate::config::*;
use validate_pgp_server::validate_certificate::sslserver::Server;
use validate_pgp_server::validate_certificate::validate::*;



/// Our pid file
static PID_FILE: &str = "/tmp/ontp-lserv.pid";



/// remove the temporary pid files
///
fn proc_rem_pid_file() 
{
   if Path::new( PID_FILE ).exists() {
      match fs::remove_file( PID_FILE) {
         Err(e) => { println!("error removing pid file '{:?}'", e);},
         Ok(_r) => { }
      };
   }
}


///
///
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> 
{
   let mut signals = Signals::new(&[ SIGHUP, SIGTERM, SIGINT, SIGQUIT, ])?;
   let mut config = Config::from_command_line().unwrap();
   loginfo(format!("validate_pgp_server starting ..."));
   //
   config.validate_report_config();
   if config.is_config_valid() == false {
      panic!("configuration is not valid exiting");
   }
   
   // write client pid file data to new file
   fs::write( PID_FILE,  format!("{:}", std::process::id()))
       .expect("Unable to write pid file");
   //
   tokio::spawn(async move {
      match Server::new(&config) {
         Ok(s)  => {
            s.server_run(config).await; 
         },
         Err(e) => { 
             loginfo(format!("validate_pgp_server ending (error launching server) ['{:?}']",e));
             proc_rem_pid_file();
             std::process::exit(2);
         }
      };
   });
   //
   let loop_end = true;
   'outer: loop {
        for signal in signals.pending() {      // Pick up signals that arrived since last time
            match signal as libc::c_int {
                SIGTERM | SIGINT | SIGQUIT => {
                    break 'outer;
                },
                _ => unreachable!(),
            }
        }
        // sleep so we don't block forever with a SIGTERM already waiting.
        thread::sleep(Duration::from_millis(500));
   }

   if loop_end {
      proc_rem_pid_file();
      loginfo(format!("validate_pgp_server ending (loop)"));
      std::process::exit(0);
   }
   Ok(())
}
//
//
