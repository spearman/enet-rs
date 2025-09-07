//! <http://enet.bespin.org/Tutorial.html>

use enet::address;

fn main() {
  println!("linked version: {}", enet::linked_version());

  println!("initializing ENet...");
  let enet = enet::initialize().unwrap();
  println!("...ENet initialized");

  println!("creating ENet server host...");
  let address = address::Address::localhost (12345);

  let _ = enet.server_host_create (
    address, // address to bind the server host to
    32,      // allow up to 32 clients and/or outgoing connections
    Some(2), // allow up to 2 channels to be used, 0 and 1
    None,    // assume any amount of incoming bandwidth
    None     // assume any amount of outgoing bandwidth
  ).unwrap();
  println!("...ENet server host created");

  println!("creating ENet client host...");
  let _ = enet.client_host_create(
    1,               // only allow 1 outgoing connection
    Some(57600 / 8), // 56 Kbps downstream bandwidth
    Some(14400 / 8)  // 14 Kbps upstream bandwidth
  ).unwrap();

  println!("...ENet client host created");
}
