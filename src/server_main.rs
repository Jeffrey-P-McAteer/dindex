use victorem;
use serde_cbor::from_slice;

use std::net::SocketAddr;
use std::time::Duration;

use dindex::get_config;
use dindex::Command;

fn main() {
  let config = get_config();
  // TODO currently victorem cannot use config.listen_ip; 
  listen(config.listen_port);
}

fn listen(port: u16) {
  let mut server = victorem::GameServer::new(ServerGlobalData { id: 0 }, port).unwrap();
  server.run();
}

struct ServerGlobalData {
    id: u32,
}

impl victorem::Game for ServerGlobalData {
    fn handle_command(
        &mut self,
        delta_time: Duration,
        commands: Vec<Vec<u8>>,
        from: SocketAddr,
    ) -> victorem::ContinueRunning {
        for v in commands {
            let cmd: Command = serde_cbor::from_slice(&v).unwrap();
            println!(
                "From Client: {:?} {} {:?}",
                delta_time,
                from,
                cmd,
            );
        }
        true
    }

    fn draw(&mut self, delta_time: Duration) -> Vec<u8> {
        self.id += 1;
        format!("Pong {} {:?}", self.id, delta_time).into_bytes()
    }
}

