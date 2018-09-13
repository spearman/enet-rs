extern crate enet;
use enet::address::Address;
use enet::version::linked_version;
use enet::Event;

/*
use enet::packet::{
  Packet,
  RELIABLE};
*/

fn main() {
  println!("server main...");

  println!("linked version: {}", linked_version());

  println!("initializing ENet...");
  let enet = enet::initialize().unwrap();
  println!("...ENet initialized");

  println!("creating ENet server host...");
  let address = Address::localhost (12345);

  let mut server = enet.server_host_create (
    address, // address to bind the server host to
    32,      // allow up to 32 clients and/or outgoing connections
    Some(2), // allow up to 2 channels to be used, 0 and 1
    None,    // assume any amount of incoming bandwidth
    None     // assume any amount of outgoing bandwidth
  ).unwrap();
  println!("...ENet server host created");

  // server loop
  println!("entering service loop...");
  loop {
    let event = server.service(10).unwrap();
    match event {
      Some(Event::Connect { .. }) => {
        println!("server received connection event:\n{:#?}", event.unwrap());
      }
      Some(Event::Disconnect { .. }) => {
        println!(
          "server received disconnection event:\n{:#?}",
          event.unwrap()
        );
      }
      Some(Event::Receive { .. }) => {
        println!("server received packet event:\n{:#?}", event.unwrap());
      }
      None => ()
    }
  }

  // unreachable
  //println!("...server main");
}
