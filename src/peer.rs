use std;
use ll;
use crate::{host, Address, Packet};

/// (65536)
pub const PACKET_LOSS_SCALE : u32 = ll::ENET_PEER_PACKET_LOSS_SCALE as u32;

////////////////////////////////////////////////////////////////////////////////
//  structs                                                                   //
////////////////////////////////////////////////////////////////////////////////

/// A type representing a connection to an ENet peer (client or server).
///
/// The first available peer (with `State::Disconnected`) is returned when
/// *connecting* to a remote server host:
///
/// ```text
/// let mut peer = host.connect (channels, data);
/// ```
///
/// Note that the peer will be able to communicate on at least as many channels
/// as the server supports, not necessarily the number requested.
/// TODO: is this reflected after a connection event is received?
///
/// A peer is an endpoint of a bi-directional channel, so any connected peer can
/// send messages:
///
/// ```text
/// peer.send (channel, packet);
/// ```
///
/// On the receiving end of the endpoint, each time an `Event` is received, a
/// reference to the corresponding peer is included.
///
/// Internally a peer is a pointer into the host's allocated array of peers.
#[derive(Clone, Debug, PartialEq)]
pub struct Peer {
  raw      : *mut ll::ENetPeer,
  hostdrop : std::rc::Rc <host::HostDrop>
}

////////////////////////////////////////////////////////////////////////////////
//  enums                                                                     //
////////////////////////////////////////////////////////////////////////////////

enum_from_primitive! {
  #[derive(Copy, Clone, Debug, Eq, PartialEq)]
  pub enum State {
    Disconnected         = ll::_ENetPeerState_ENET_PEER_STATE_DISCONNECTED
      as isize,
    Connecting           = ll::_ENetPeerState_ENET_PEER_STATE_CONNECTING
      as isize,
    AcknowledgingConnect = ll::_ENetPeerState_ENET_PEER_STATE_ACKNOWLEDGING_CONNECT
      as isize,
    ConnectionPending    = ll::_ENetPeerState_ENET_PEER_STATE_CONNECTION_PENDING
      as isize,
    ConnectionSucceeded  = ll::_ENetPeerState_ENET_PEER_STATE_CONNECTION_SUCCEEDED
      as isize,
    Connected            = ll::_ENetPeerState_ENET_PEER_STATE_CONNECTED
      as isize,
    DisconnectLater      = ll::_ENetPeerState_ENET_PEER_STATE_DISCONNECT_LATER
      as isize,
    Disconnecting        = ll::_ENetPeerState_ENET_PEER_STATE_DISCONNECTING
      as isize,
    AcknowledgingDisconnect = ll::_ENetPeerState_ENET_PEER_STATE_ACKNOWLEDGING_DISCONNECT
      as isize,
    Zombie               = ll::_ENetPeerState_ENET_PEER_STATE_ZOMBIE
      as isize
  }
}

#[derive(Debug)]
pub enum ConnectError {
  NoPeersAvailable,
  /// failure due to internal malloc failure of channel allocation
  Failure
}

#[derive(Debug)]
pub enum SendError {
  PeerNotConnected (State),
  PeerNoChannelID (u8),
  PacketCreateZeroLength,
  /// packet creation failed due to internal malloc call failing
  PacketCreateMallocFailure,
  /// packet size exceeds raw value of `peer.host->maximumPacketSize`
  PacketExceedsMaximumSize (usize),
  /// failure due to either malloc failure or internal fragment count
  /// exceeded `ENET_PROTOCOL_MAXIMUM_FRAGMENT_COUNT`
  Failure
}

////////////////////////////////////////////////////////////////////////////////
//  impls                                                                     //
////////////////////////////////////////////////////////////////////////////////

impl Peer {
  pub (crate) unsafe fn from_raw (
    peer     : *mut ll::ENetPeer,
    hostdrop : std::rc::Rc <host::HostDrop>
  ) -> Self {
    Peer {
      raw: peer,
      hostdrop
    }
  }

  #[inline]
  pub unsafe fn raw (&self) -> *mut ll::ENetPeer {
    self.raw
  }

  /// The `incomingPeerID` field represents the index into the local array of
  /// peers.
  ///
  /// Note this is *not* the same as the `connectID` field.
  #[inline]
  pub fn incoming_peer_id (&self) -> u16 {
    unsafe { (*self.raw).incomingPeerID }
  }

  #[inline]
  pub fn state (&self) -> State {
    use enum_primitive::FromPrimitive;
    unsafe { State::from_u32 ((*self.raw).state as u32).unwrap() }
  }

  #[inline]
  pub fn address (&self) -> Address {
    unsafe { Address::from_ll ((*self.raw).address) }
  }

  #[inline]
  pub fn packet_loss_epoch (&self) -> u32 {
    unsafe { (*self.raw).packetLossEpoch }
  }

  /// Packets sent during the current "packet loss epoch"
  #[inline]
  pub fn packets_sent (&self) -> u32 {
    unsafe { (*self.raw).packetsSent }
  }

  /// Packets lost during the current "packet loss epoch"
  #[inline]
  pub fn packets_lost (&self) -> u32 {
    unsafe { (*self.raw).packetsLost }
  }

  /// Mean packet loss of reliable packets as a ratio with respect to
  /// `PACKET_LOSS_SCALE`
  #[inline]
  pub fn packet_loss (&self) -> u32 {
    unsafe { (*self.raw).packetLoss }
  }

  /// Milliseconds
  #[inline]
  pub fn round_trip_time (&self) -> u32 {
    unsafe { (*self.raw).roundTripTime }
  }

  /// Milliseconds
  #[inline]
  pub fn round_trip_time_variance (&self) -> u32 {
    unsafe { (*self.raw).roundTripTimeVariance }
  }

  // TODO: expose the following round trip time values ?
  /*
  /// Milliseconds
  #[inline]
  pub fn lowest_round_trip_time (&self) -> u32 {
    unsafe { (*self.raw).lowestRoundTripTime }
  }

  /// Milliseconds
  #[inline]
  pub fn highest_round_trip_time_variance (&self) -> u32 {
    unsafe { (*self.raw).highestRoundTripTimeVariance }
  }

  /// Milliseconds
  #[inline]
  pub fn last_round_trip_time (&self) -> u32 {
    unsafe { (*self.raw).lastRoundTripTime }
  }

  /// Milliseconds
  #[inline]
  pub fn last_round_trip_time_variance (&self) -> u32 {
    unsafe {
      (*self.raw).lastRoundTripTimeVariance
    }
  }
  */

  #[inline]
  pub fn ping (&mut self) {
    unsafe {
      ll::enet_peer_ping (self.raw())
    }
  }

  #[inline]
  pub fn get_ping_interval (&mut self) -> u32 {
    unsafe {
      (*self.raw).pingInterval
    }
  }

  /// Set the interval at which pings will be sent to a peer
  #[inline]
  pub fn ping_interval (&mut self, ping_interval : u32) {
    unsafe {
      ll::enet_peer_ping_interval (self.raw(), ping_interval)
    }
  }

  /// Set the timeout parameters for a peer
  #[inline]
  pub fn timeout (&mut self,
    timeout_limit : u32, timeout_minimum : u32, timeout_maximum : u32
  ) {
    unsafe {
      ll::enet_peer_timeout (self.raw(),
        timeout_limit, timeout_minimum, timeout_maximum)
    }
  }

  /// (timeoutLimit, timeoutMinimum, timeoutMaximum)
  #[inline]
  pub fn get_timeout (&mut self) -> (u32, u32, u32) {
    unsafe {
      ( (*self.raw).timeoutLimit,
        (*self.raw).timeoutMinimum,
        (*self.raw).timeoutMaximum )
    }
  }

  pub fn send (&mut self, channel_id : u8, packet : Packet)
    -> Result <(), SendError>
  {
    use enum_primitive::FromPrimitive;
    unsafe {
      if (*self.raw).state != ll::_ENetPeerState_ENET_PEER_STATE_CONNECTED {
        return Err (SendError::PeerNotConnected(
          State::from_u32 ((*self.raw).state as u32).unwrap()
        ))
      }
      if (*self.raw).channelCount as u8 <= channel_id {
        return Err (SendError::PeerNoChannelID (channel_id))
      }
      let raw;
      match packet {
        Packet::Allocate { bytes, flags } => {
          if (*self.hostdrop.raw()).maximumPacketSize < bytes.len() as usize {
            return Err (SendError::PacketExceedsMaximumSize (bytes.len()))
          }
          if bytes.is_empty() {
            return Err (SendError::PacketCreateZeroLength)
          }
          raw = ll::enet_packet_create (
            bytes.as_ptr() as *const std::os::raw::c_void,
            bytes.len() as usize,
            flags.bits()
          );
        }
        Packet::NoAllocate { bytes, flags } => {
          if (*(*self.raw).host).maximumPacketSize < bytes.len() as usize {
            return Err (SendError::PacketExceedsMaximumSize(bytes.len()))
          }
          if bytes.is_empty() {
            return Err (SendError::PacketCreateZeroLength)
          }
          raw = ll::enet_packet_create (
            bytes.as_ptr() as *const std::os::raw::c_void,
            bytes.len() as usize,
            flags.bits() | ll::_ENetPacketFlag_ENET_PACKET_FLAG_NO_ALLOCATE
              as u32
          );
        }
      }
      if raw.is_null() {
        return Err (SendError::PacketCreateMallocFailure)
      }
      if ll::enet_peer_send (self.raw(), channel_id, raw) < 0 {
        return Err (SendError::Failure)
      }
      Ok(())
    }
  } // end send

  // TODO: expose data parameter in the following ?

  /// Request a disconnection.
  ///
  /// Note this may be sent before queued outgoing packets; use
  /// `disconnect_later` to ensure they are sent before disconnecting.
  #[inline]
  pub fn disconnect (&self) {
    unsafe {
      ll::enet_peer_disconnect (self.raw(), 0)
    }
  }

  /// Force immediate disconnection
  #[inline]
  pub fn disconnect_now (&self) {
    unsafe {
      ll::enet_peer_disconnect_now (self.raw(), 0)
    }
  }

  /// Request disconnection after all queued outgoing packets are sent
  #[inline]
  pub fn disconnect_later (&self) {
    unsafe {
      ll::enet_peer_disconnect_later (self.raw(), 0)
    }
  }

  /// Forcibly disconnect without notifying peer.
  ///
  /// For the remote peer, eventually the connection will time out.
  #[inline]
  pub fn reset (&self) {
    unsafe {
      ll::enet_peer_reset (self.raw())
    }
  }

} // end impl Peer
