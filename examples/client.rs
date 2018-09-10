extern crate enet;
use enet::address::Address;
use enet::packet::{Packet, PacketFlags};
use enet::version::linked_version;
use enet::Event;

fn main() {
  println!("client main...");

  println!("linked version: {}", linked_version());

  println!("initializing ENet...");
  let enet = enet::initialize().unwrap();
  println!("...ENet initialized");

  println!("creating ENet client host...");
  let mut client = enet
    .client_host_create(
      1,               // only allow 1 outgoing connection
      Some(57600 / 8), // 56 Kbps downstream bandwidth
      Some(14400 / 8)  // 14 Kbps upstream bandwidth
    ).unwrap();
  println!("...ENet client host created");

  // server address
  let server_address = Address::with_hostname("127.0.0.1", 12345).unwrap();

  println!("client connecting to server...");
  let mut server_peer = client.connect(&server_address, 2, 0).unwrap();

  // wait for connection
  let connected = client.service(5000).unwrap();
  match connected {
    Some(Event::Connect { .. }) => println!("...client connected to server"),
    _ => panic!("client connection to server failed")
  }

  // send packet
  println!("sending packet to server...");
  let packet = Packet::Allocate {
    bytes : "abc".as_bytes(),
    flags : PacketFlags::RELIABLE
  };

  server_peer.send(0, packet).unwrap();
  println!("...packet sent");

  // client loop
  println!("entering service loop...");
  loop {
    let event = client.service(10).unwrap();

    //FIXME:
    match event {
      Some(Event::Connect { .. }) => {
        println!("client received connection event:\n{:#?}", event.unwrap());
      }
      Some(Event::Disconnect { .. }) => {
        println!(
          "client received disconnection event:\n{:#?}",
          event.unwrap()
        );
      }
      Some(Event::Receive { .. }) => {
        println!("client received packet event:\n{:#?}", event.unwrap());
      }
      None => ()
    }
  }

  // unreachable
  //println!("...client main");
}
