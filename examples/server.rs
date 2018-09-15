extern crate ctrlc;
extern crate enet;

use std::sync::atomic;

use enet::*;

macro_rules! show {
  ($e:expr) => {
    println!("{}: {:?}", stringify!($e), $e);
  }
}

const PORT       : u16 = 12345;
const SERVICE_MS : u32 = 500;

fn main() {
  println!("example server main...");

  println!("starting SIGINT handler...");
  let running       = std::sync::Arc::new (atomic::AtomicBool::new (true));
  let running_clone = running.clone();
  ctrlc::set_handler (move || {
    println!("\ngot SIGINT: closing...");
    running_clone.store (false, atomic::Ordering::SeqCst)
  }).unwrap();

  println!("initializing ENet...");
  show!(version::linked_version());
  let enet = initialize().unwrap();
  println!("...ENet initialized");
  println!("creating ENet server host...");
  let address = Address::localhost (PORT);
  let mut server = enet.server_host_create (
    address,    // address to bind the server host to
    32,         // allow up to 32 clients and/or outgoing connections
    Some (2),   // allow up to 2 channels to be used, 0 and 1
    None, None, // assume any amount of incoming & outgoing bandwidth
  ).unwrap();
  println!("...ENet server host created");

  // service loop
  println!("awaiting connections on port {}", PORT);
  let mut client_peer = None;
  let mut iter        = 0;
  println!("entering service loop...");
  while running.load (atomic::Ordering::SeqCst) {
    // DEBUG: sleep for a long time
    /*
    if iter % 20 == 19 {
      std::thread::sleep (std::time::Duration::from_secs (40));
    }
    */
    show!((iter, client_peer.as_ref().map (Peer::state)));
    match server.service (SERVICE_MS) {
      Ok (Some (event @ Event::Connect    {..})) => {
        println!("server received connection event:\n{:#?}",    event);
        if let Event::Connect { peer, .. } = event {
          if client_peer.is_none() {
            client_peer = Some (peer.clone());
          }
        } else { unreachable!() }
      }
      Ok (Some (event @ Event::Disconnect {..})) => {
        println!("server received disconnection event:\n{:#?}", event);
        client_peer = None;
      }
      Ok (Some (event @ Event::Receive    {..})) =>
        println!("server received packet event:\n{:#?}",        event),
      Ok  (None) => {}
      Err (err)  => println!("service error: {:?}", err)
    }
    iter += 1;
  }

  println!("...example server main");
}
