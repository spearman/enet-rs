//! Rust interface for the [ENet reliable UDP library](http://enet.bespin.org/)

#![feature(int_to_from_bytes)]

#[macro_use] extern crate bitflags;
#[macro_use] extern crate enum_primitive;

extern crate enet_sys as ll;

use std::sync::atomic;

pub mod address;
pub mod event;
pub mod host;
pub mod packet;
pub mod peer;
pub mod version;

pub use self::address::Address;
pub use self::event::Event;
pub use self::host::Host;
pub use self::packet::Packet;
pub use self::peer::Peer;
pub use self::version::Version;

/// (4096)
pub const MAX_PEERS         : u32 = ll::ENET_PROTOCOL_MAXIMUM_PEER_ID;
/// (255)
pub const MAX_CHANNEL_COUNT : u32 = ll::ENET_PROTOCOL_MAXIMUM_CHANNEL_COUNT;

static ENET_CONTEXT_ALIVE : atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

/// The initialized ENet context.
///
/// This can be sent accross threads and used to create thread-local `Host`s.
///
/// The context will be kept alive as long as any `Host`s still exist.
#[derive(Clone)]
pub struct Enet {
  enetdrop : std::sync::Arc <EnetDrop>
}

#[derive(Clone, Debug, PartialEq)]
struct EnetDrop;

/// ENet context errors
#[derive(Clone, Debug)]
pub enum Error {
  Initialize   (String),
  ServerCreate (host::CreateError),
  ClientCreate (host::CreateError)
}

/// Initialize the ENet context.
///
/// An error is returned if ENet is already initialized.
#[inline]
pub fn initialize() -> Result <Enet, Error> {
  Enet::new()
}

/// Safe to call regardless of ENet initialization
pub fn linked_version() -> Version {
  unsafe {
    Version::from_ll (ll::enet_linked_version())
  }
}

impl Enet {
  /// Create a host that is intended to only request new connections and not to
  /// listen for incoming connections.
  ///
  /// Bandwidth parameters determine the "window size" of a connection which
  /// limits the number of reliable packets that may be in transit at any given
  /// time.
  pub fn client_host_create (&self,
    peer_count         : u32,
    incoming_bandwidth : Option <u32>,
    outgoing_bandwidth : Option <u32>
  ) -> Result <Host, Error> {
    Host::new (
      None,
      peer_count,
      None,
      incoming_bandwidth,
      outgoing_bandwidth,
      self.enetdrop.clone()
    ).map_err (Error::ClientCreate)
  }

  /// Create a host that is intended to listen for incoming connections (and may
  /// also request new connections with remote hosts).
  pub fn server_host_create (&self,
    address            : Address,
    peer_count         : u32,
    channel_limit      : Option <u32>,
    incoming_bandwidth : Option <u32>,
    outgoing_bandwidth : Option <u32>
  ) -> Result <Host, Error> {
    Host::new (
      Some (address),
      peer_count,
      channel_limit,
      incoming_bandwidth,
      outgoing_bandwidth,
      self.enetdrop.clone()
    ).map_err (Error::ServerCreate)
  }

  fn new() -> Result <Self, Error> {
    unsafe {
      let was_alive =
        ENET_CONTEXT_ALIVE.swap (true, atomic::Ordering::Relaxed);
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
}

impl Drop for EnetDrop {
  #[inline]
  fn drop (&mut self) {
    let was_alive = ENET_CONTEXT_ALIVE.swap (false, atomic::Ordering::Relaxed);
    debug_assert!(was_alive);
    if was_alive {
      unsafe { ll::enet_deinitialize() }
    }
  }
}

////////////////////////////////////////////////////////////////////////////////
//  tests                                                                     //
////////////////////////////////////////////////////////////////////////////////

/*
#[cfg (test)]
mod tests {
  #[test]
  fn it_works() {}
}
*/
