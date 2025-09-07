use std;
use ll;
use crate::{
  peer, Address, EnetDrop, Event, Packet, Peer, MAX_PEERS, MAX_CHANNEL_COUNT
};

////////////////////////////////////////////////////////////////////////////////
//  structs                                                                   //
////////////////////////////////////////////////////////////////////////////////

/// An ENet host for communicating with peers.
///
/// A `Host` cannot be sent accross threads but will keep Enet alive.
#[derive(Clone, Debug)]
pub struct Host {
  hostdrop : std::rc::Rc <HostDrop>
}

#[derive(Debug, PartialEq)]
pub(crate) struct HostDrop {
  raw      : *mut ll::ENetHost,
  enetdrop : std::sync::Arc <EnetDrop>
}

////////////////////////////////////////////////////////////////////////////////
//  enums                                                                     //
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum Error {
  /// Error from `service()`
  ServiceError,
  /// Error from `check_events()`
  DispatchError
}

#[derive(Clone, Debug)]
pub enum CreateError {
  /// Maximum peer count is `enet::MAX_PEERS` (4096)
  TooManyPeers    (u32),
  /// Maximum channel count is `enet::MAX_CHANNEL_COUNT` (255)
  TooManyChannels (u32),
  ReturnedNull
}

////////////////////////////////////////////////////////////////////////////////
//  impls                                                                     //
////////////////////////////////////////////////////////////////////////////////

impl Host {
  pub(crate) fn new (
    address            : Option <Address>,
    peer_count         : u32,
    channel_limit      : Option <u32>,
    incoming_bandwidth : Option <u32>,
    outgoing_bandwidth : Option <u32>,
    enetdrop           : std::sync::Arc <EnetDrop>
  ) -> Result <Self, CreateError> {
    if MAX_PEERS < peer_count {
      return Err (CreateError::TooManyPeers (peer_count))
    }
    let channel_limit = channel_limit.unwrap_or (0);
    if MAX_CHANNEL_COUNT < channel_limit {
      return Err (CreateError::TooManyChannels (channel_limit))
    }
    let host;
    match address {
      Some (a) => unsafe {
        host = ll::enet_host_create (
          a.raw(),
          peer_count    as usize,
          channel_limit as usize,
          incoming_bandwidth.unwrap_or (0),
          outgoing_bandwidth.unwrap_or (0)
        );
        if host.is_null() {
          return Err (CreateError::ReturnedNull)
        }
      },
      None => unsafe {
        host = ll::enet_host_create (
          std::ptr::null(),
          peer_count    as usize,
          channel_limit as usize,
          incoming_bandwidth.unwrap_or (0),
          outgoing_bandwidth.unwrap_or (0)
        );
        if host.is_null() {
          return Err (CreateError::ReturnedNull)
        }
      }
    } // end match address
    Ok (Host {
      hostdrop: std::rc::Rc::new (HostDrop {
        raw: host, enetdrop
      })
    })
  } // end new

  /// # Safety
  ///
  /// Unsafe: returns raw pointer.
  #[inline]
  pub unsafe fn raw (&self) -> *mut ll::ENetHost {
    unsafe { self.hostdrop.raw() }
  }

  /// Number of peers allocated for this host
  #[inline]
  pub fn peer_count (&self) -> usize {
    unsafe { (*self.raw()).peerCount }
  }

  /// Number of connected peers
  #[inline]
  pub fn connected_peers (&self) -> usize {
    unsafe { (*self.raw()).connectedPeers }
  }

  /// Maximum number of channels for incoming connections
  #[inline]
  pub fn channel_limit (&self) -> usize {
    unsafe { (*self.raw()).channelLimit }
  }

  /// Total UDP packets sent.
  ///
  /// User must reset to prevent overflow.
  #[inline]
  pub fn total_sent_packets (&self) -> u32 {
    unsafe { (*self.raw()).totalSentPackets }
  }
  pub fn reset_total_sent_packets (&mut self) {
    unsafe {
      (*self.raw()).totalSentPackets = 0;
    }
  }

  /// Total bytes sent.
  ///
  /// User must reset to prevent overflow.
  #[inline]
  pub fn total_sent_data (&self) -> u32 {
    unsafe { (*self.raw()).totalSentPackets }
  }
  pub fn reset_total_sent_data (&mut self) {
    unsafe {
      (*self.raw()).totalSentData = 0;
    }
  }

  /// Total UDP packets received.
  ///
  /// User must reset to prevent overflow.
  #[inline]
  pub fn total_received_packets (&self) -> u32 {
    unsafe { (*self.raw()).totalReceivedPackets }
  }
  pub fn reset_total_received_packets (&mut self) {
    unsafe {
      (*self.raw()).totalReceivedPackets = 0;
    }
  }

  /// Total bytes received.
  ///
  /// User must reset to prevent overflow.
  #[inline]
  pub fn total_received_data (&self) -> u32 {
    unsafe { (*self.raw()).totalReceivedPackets }
  }
  pub fn reset_total_received_data (&mut self) {
    unsafe {
      (*self.raw()).totalReceivedData = 0;
    }
  }


  /// Initiate a connection with a remote host.
  ///
  /// When connecting to a peer with the `host.connect()` method, a `Peer` representing
  /// the connection will be created in the `PeerState::Connecting` state:
  /// ```
  /// # use enet::Address;
  /// # let enet = enet::initialize().unwrap();
  /// # let mut client = enet.client_host_create (1, None, None).unwrap();
  /// let mut peer = client.connect (&Address::localhost (12345), 2, 0);
  /// ```
  /// where the second argument (`2`) is the number of channels to allocate to
  /// the connection and the third argument (`0`) is an internal `data : u32`
  /// that can be used by the application.
  ///
  /// After receipt of a `Connect` event, the peer is ready to use.
  ///
  /// *Note*: after receipt of the `Connect` event on the host that originated
  /// the connection request, a call to `flush()` or `service()` is required to
  /// *acknowledge* the connection has succeeded in order to generate the
  /// corresponding `Connect` event on the server end.
  ///
  /// That connection will now be 'in use' until the peer is changed to the
  /// `PeerState::Disconnected` state.
  ///
  /// Note that `Host`s can connect *mutually* (host A connected to host B, and
  /// host B connected to host A), or *multiply* (host A connected to host B
  /// more than 1 time), and each connection will have its own `Peer` structure
  /// in each host A and B.
  pub fn connect (&mut self, address : &Address, channel_count : u8, data : u32)
    -> Result <Peer, peer::ConnectError>
  {
    unsafe {
      if self.peer_count() <= self.connected_peers() {
        return Err (peer::ConnectError::NoPeersAvailable)
      }
      let peer = ll::enet_host_connect (
        self.raw(),
        address.raw(),
        channel_count as usize,
        data
      );
      if peer.is_null() {
        return Err (peer::ConnectError::Failure)
      }
      Ok (Peer::from_raw(peer, self.hostdrop.clone()))
    }
  }

  /// Waits for events on the host specified and shuttles packets between the
  /// host and its peers. Sends queued messages and dispatches events.
  /// Alternatively, `flush()` will send queued messages without dispatching
  /// events.
  ///
  /// `timeout` is the number of milliseconds that ENet should wait for events.
  pub fn service (&mut self, timeout : u32) -> Result <Option <Event>, Error> {
    let event = unsafe {
      let mut mem = std::mem::MaybeUninit::<ll::ENetEvent>::uninit();
      let event   = mem.as_mut_ptr();
      if ll::enet_host_service (self.hostdrop.raw, event, timeout) < 0 {
        return Err (Error::ServiceError)
      }
      *event
    };
    Ok (Event::from_ll (event, self.hostdrop.clone()))
  }

  /// Checks for any queued events on the host and dispatches one if available
  #[inline]
  pub fn check_events (&mut self) -> Result <Option <Event>, Error> {
    let event = unsafe {
      let mut mem = std::mem::MaybeUninit::<ll::ENetEvent>::uninit();
      let event   = mem.as_mut_ptr();
      if ll::enet_host_check_events (self.hostdrop.raw, event) < 0 {
        return Err (Error::DispatchError)
      }
      *event
    };
    Ok (Event::from_ll (event, self.hostdrop.clone()))
  }

  /// Send any queued messages without dispatching events. Alternatively,
  /// `service()` will send queued messages and also dispatch events.
  #[inline]
  pub fn flush (&mut self) {
    unsafe { ll::enet_host_flush (self.hostdrop.raw) }
  }

  /// Queue a packet to be sent to all peers associated with the host
  pub fn broadcast (&mut self, channel_id : u8, packet : Packet) {
    unsafe {
      let raw = match packet {
        Packet::Allocate { bytes, flags } => {
          ll::enet_packet_create (
            bytes.as_ptr() as *const std::os::raw::c_void,
            bytes.len(),
            flags.bits())
        }
        #[expect(clippy::unnecessary_cast)]  // NOTE: on windows ll flags are i32
        Packet::NoAllocate { bytes, flags } => {
          ll::enet_packet_create (
            bytes.as_ptr() as *const std::os::raw::c_void,
            bytes.len(),
            flags.bits() | ll::_ENetPacketFlag_ENET_PACKET_FLAG_NO_ALLOCATE as u32)
        }
      };
      ll::enet_host_broadcast (self.raw(), channel_id, raw)
    }
  }

} // end impl Host

impl HostDrop {
  #[inline]
  pub(crate) const unsafe fn raw (&self) -> *mut ll::ENetHost {
    self.raw
  }
}
impl Drop for HostDrop {
  #[inline]
  fn drop (&mut self) {
    unsafe { ll::enet_host_destroy (self.raw) }
  }
}
