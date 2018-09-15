use {std, ll};
use {host, Packet};

////////////////////////////////////////////////////////////////////////////////
//  structs                                                                   //
////////////////////////////////////////////////////////////////////////////////

/// A type representing a connection to an ENet peer (client or server).
///
/// The first available peer (with `PeerState::Disconnected`) is returned when
/// *connecting* to a remote server host:
///
/// ```text
/// let mut peer = host.connect (CHANNELS, DATA);
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
/// peer.send (CHANNEL, packet);
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
  pub enum PeerState {
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
pub enum SendError {
  PeerNotConnected (PeerState),
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
  pub unsafe fn from_raw (
    peer     : *mut ll::ENetPeer,
    hostdrop : std::rc::Rc <host::HostDrop>
  ) -> Self {
    Peer {
      raw : peer,
      hostdrop
    }
  }

  #[inline]
  pub unsafe fn raw (&self) -> *mut ll::ENetPeer {
    self.raw
  }

  #[inline]
  pub fn state (&self) -> PeerState {
    use enum_primitive::FromPrimitive;
    unsafe {
      PeerState::from_u32 ((*self.raw).state).unwrap()
    }
  }

  #[inline]
  pub fn disconnect (&self) {
    unsafe {
      ll::enet_peer_disconnect (self.raw(), 0)
    }
  }

  /// Forcibly disconnect without notifying peer.
  ///
  /// For the peer, eventually the connection will time out.
  #[inline]
  pub fn reset (&self) {
    unsafe {
      ll::enet_peer_reset (self.raw())
    }
  }

  pub fn send (&mut self, channel_id : u8, packet : Packet)
    -> Result <(), SendError>
  {
    use enum_primitive::FromPrimitive;
    unsafe {
      if (*self.raw).state != ll::_ENetPeerState_ENET_PEER_STATE_CONNECTED {
        return Err (SendError::PeerNotConnected(
          PeerState::from_u32 ((*self.raw).state).unwrap()
        ))
      }
      if (*self.raw).channelCount as u8 <= channel_id {
        return Err (SendError::PeerNoChannelID(channel_id))
      }
      let raw;
      match packet {
        Packet::Allocate { bytes, flags } => {
          if (*self.hostdrop.raw()).maximumPacketSize < bytes.len() {
            return Err (SendError::PacketExceedsMaximumSize (bytes.len()))
          }
          if bytes.is_empty() {
            return Err (SendError::PacketCreateZeroLength)
          }
          raw = ll::enet_packet_create(
            bytes.as_ptr() as (*const std::os::raw::c_void),
            bytes.len(),
            flags.bits()
          );
        }
        Packet::NoAllocate { bytes, flags } => {
          if (*(*self.raw).host).maximumPacketSize < bytes.len() {
            return Err (SendError::PacketExceedsMaximumSize(bytes.len()))
          }
          if bytes.is_empty() {
            return Err (SendError::PacketCreateZeroLength)
          }
          raw = ll::enet_packet_create(
            bytes.as_ptr() as (*const std::os::raw::c_void),
            bytes.len(),
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
} // end impl Peer
