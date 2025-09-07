use std::sync::atomic;

use ctrlc;
use enet::*;

macro_rules! show {
  ($e:expr) => {
    println!("{}: {:?}", stringify!($e), $e);
  }
}

const PORT       : u16 = 12345;
const SERVICE_MS : u32 = 500;

fn main() {
  println!("example client main...");

  println!("starting SIGINT handler...");
  let running       = std::sync::Arc::new (atomic::AtomicBool::new (true));
  let running_clone = running.clone();
  ctrlc::set_handler (move || {
    println!("\ngot SIGINT: closing...");
    running_clone.store (false, atomic::Ordering::SeqCst)
  }).unwrap();

  println!("initializing ENet...");
  show!(linked_version());
  let enet = initialize().unwrap();
  println!("...ENet initialized");
  println!("creating ENet client host...");
  let mut client = enet.client_host_create (
    1,                // only allow 1 outgoing connection
    Some (57600 / 8), // 56 Kbps downstream bandwidth
    Some (14400 / 8)  // 14 Kbps upstream bandwidth
  ).unwrap();
  println!("...ENet client host created");
  let server_address  = Address::localhost (PORT);
  let mut server_peer = client.connect (&server_address, 2, 0).unwrap();
  println!("client connecting to server at: {server_address:?}");
  show!(server_peer.state());
  // wait up to 5 seconds for connection
  match client.service (5000).unwrap() {
    Some (event @ Event::Connect { .. })
      => println!("got connection event: {event:#?}"),
    _ => panic!("client connection to server failed")
  }
  show!(server_peer.state());
  client.flush();   // send connection ack to server
  println!("...client connected to server");

  // send a packet
  println!("sending packet to server...");
  let packet = Packet::Allocate {
    bytes: "abc".as_bytes(),
    flags: packet::Flags::RELIABLE
  };
  server_peer.send (0, packet).unwrap();
  client.flush();   // send packet
  println!("...packet sent");

  // client loop
  println!("entering service loop...");
  let mut iter = 0;
  while running.load (atomic::Ordering::SeqCst) {
    show!((iter, server_peer.state()));
    match client.service (SERVICE_MS) {
      Ok (Some (event @ Event::Connect    {..})) =>
        println!("client received connection event:\n{event:#?}"),
      Ok (Some (event @ Event::Disconnect {..})) => {
        println!("client received disconnection event:\n{event:#?}");
        running.store (false, atomic::Ordering::SeqCst)
      }
      Ok (Some (event @ Event::Receive    {..})) =>
        println!("client received packet event:\n{event:#?}"),
      Ok (None) => {}
      Err (err) => println!("client received error: {err:?}")
    }
    iter += 1;
  }

  server_peer.disconnect();
  client.flush();

  println!("...client main");
}
