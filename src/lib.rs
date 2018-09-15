#[macro_use] extern crate bitflags;
#[macro_use] extern crate enum_primitive;

extern crate enet_sys as ll;

use std::sync::atomic;

pub mod address;
pub mod host;
pub mod packet;
pub mod peer;
pub mod version;

pub use self::address::Address;
pub use self::host::Host;
pub use self::packet::Packet;
pub use self::peer::Peer;
pub use self::version::Version;

////////////////////////////////////////////////////////////////////////////////
//  statics                                                                   //
////////////////////////////////////////////////////////////////////////////////

static IS_ENET_CONTEXT_ALIVE : atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

////////////////////////////////////////////////////////////////////////////////
//  structs                                                                   //
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Enet {
  enetdrop : std::sync::Arc <EnetDrop>
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnetDrop;

////////////////////////////////////////////////////////////////////////////////
//  enums                                                                     //
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum Error {
  Initialize   (String),
  ServerCreate (host::CreateError),
  ClientCreate (host::CreateError)
}

#[derive(Debug)]
pub enum Event {
  Connect {
    peer : Peer,
    data : u32
  },
  Disconnect {
    peer : Peer,
    data : u32
  },
  Receive {
    peer       : Peer,
    channel_id : u8,
    packet     : packet::PacketRecv
  }
}

////////////////////////////////////////////////////////////////////////////////
//  functions                                                                 //
////////////////////////////////////////////////////////////////////////////////

#[inline]
pub fn initialize() -> Result <Enet, Error> {
  Enet::new()
}

////////////////////////////////////////////////////////////////////////////////
//  impls                                                                     //
////////////////////////////////////////////////////////////////////////////////

impl Enet {
  fn new() -> Result <Self, Error> {
    unsafe {
      let was_alive =
        IS_ENET_CONTEXT_ALIVE.swap (true, atomic::Ordering::Relaxed);
      if was_alive {
        return Err (Error::Initialize (
          "`Enet` cannot be initialized more than once".to_owned()))
      }
      if ll::enet_initialize() < 0 {
        return Err (Error::Initialize(
          "`enet_initialize` returned an error".to_owned()))
      }
      Ok (Enet { enetdrop: std::sync::Arc::new (EnetDrop) })
    }
  }

  pub fn client_host_create (&self,
    peer_count         : u32,
    incoming_bandwidth : Option <u32>,
    outgoing_bandwidth : Option <u32>
  ) -> Result <Host, Error> {
    Ok (try!(
      Host::new (
        None,
        peer_count,
        None,
        incoming_bandwidth,
        outgoing_bandwidth,
        self.enetdrop.clone()
      ).map_err (Error::ClientCreate)
    ))
  }

  pub fn server_host_create (&self,
    address            : Address,
    peer_count         : u32,
    channel_limit      : Option <usize>,
    incoming_bandwidth : Option <u32>,
    outgoing_bandwidth : Option <u32>
  ) -> Result <Host, Error> {
    Ok (try!(
      Host::new(
        Some (address),
        peer_count,
        channel_limit,
        incoming_bandwidth,
        outgoing_bandwidth,
        self.enetdrop.clone()
      ).map_err (Error::ServerCreate)
    ))
  }
} // end impl Enet

impl Drop for EnetDrop {
  #[inline]
  fn drop (&mut self) {
    let was_alive =
      IS_ENET_CONTEXT_ALIVE.swap (false, atomic::Ordering::Relaxed);
    assert!(was_alive);

    unsafe { ll::enet_deinitialize() }
  }
}

////////////////////////////////////////////////////////////////////////////////
//  tests                                                                     //
////////////////////////////////////////////////////////////////////////////////

#[cfg (test)]
mod tests {
  #[test]
  fn it_works() {}
}
