use {host, Packet};
use {ll, std};

////////////////////////////////////////////////////////////////////////////////
//  structs                                                                   //
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct Peer {
  raw :      *mut ll::ENetPeer,
  hostdrop : std::rc::Rc<host::HostDrop>
}

////////////////////////////////////////////////////////////////////////////////
//  enums                                                                     //
////////////////////////////////////////////////////////////////////////////////

enum_from_primitive! {
  #[derive(Debug)]
  pub enum PeerState {
    Disconnected = 0,
    Connecting,
    AcknowledgingConnect,
    ConnectionPending,
    ConnectionSucceeded,
    Connected,
    DisconnectLater,
    Disconnecting,
    AcknowledgingDisconnect,
    Zombie
  }
}

#[derive(Debug)]
pub enum SendError {
  PeerNotConnected(PeerState),
  PeerNoChannelID(u8),
  PacketCreateZeroLength,
  /// packet creation failed due to internal malloc call failing
  PacketCreateMallocFailure,
  /// packet size exceeds raw value of `peer.host->maximumPacketSize`
  PacketExceedsMaximumSize(usize),
  /// failure due to either malloc failure or internal fragment count
  /// exceeded `ENET_PROTOCOL_MAXIMUM_FRAGMENT_COUNT`
  Failure
}

////////////////////////////////////////////////////////////////////////////////
//  impls                                                                     //
////////////////////////////////////////////////////////////////////////////////

impl Peer {
  pub unsafe fn from_raw(
    peer : *mut ll::ENetPeer,
    hostdrop : std::rc::Rc<host::HostDrop>
  ) -> Self {
    Peer {
      raw : peer,
      hostdrop
    }
  }

  #[inline]
  pub unsafe fn raw(&self) -> *mut ll::ENetPeer {
    self.raw
  }

  pub fn send(
    &mut self,
    channel_id : u8,
    packet : Packet
  ) -> Result<(), SendError> {
    use enum_primitive::FromPrimitive;
    unsafe {
      if (*self.raw).state != ll::_ENetPeerState_ENET_PEER_STATE_CONNECTED {
        return Err(SendError::PeerNotConnected(
          PeerState::from_isize((*self.raw).state as isize).unwrap()
        ))
      }
      if (*self.raw).channelCount as u8 <= channel_id {
        return Err(SendError::PeerNoChannelID(channel_id))
      }
      let raw;
      match packet {
        Packet::Allocate { bytes, flags } => {
          if (*self.hostdrop.raw()).maximumPacketSize < bytes.len() {
            return Err(SendError::PacketExceedsMaximumSize(bytes.len()))
          }
          if bytes.is_empty() {
            return Err(SendError::PacketCreateZeroLength)
          }
          raw = ll::enet_packet_create(
            bytes.as_ptr() as (*const std::os::raw::c_void),
            bytes.len(),
            flags.bits()
          );
        }
        Packet::NoAllocate { bytes, flags } => {
          if (*(*self.raw).host).maximumPacketSize < bytes.len() {
            return Err(SendError::PacketExceedsMaximumSize(bytes.len()))
          }
          if bytes.is_empty() {
            return Err(SendError::PacketCreateZeroLength)
          }
          raw = ll::enet_packet_create(
            bytes.as_ptr() as (*const std::os::raw::c_void),
            bytes.len(),
            flags.bits()
              | ll::_ENetPacketFlag_ENET_PACKET_FLAG_NO_ALLOCATE as u32
          );
        }
      }
      if raw.is_null() {
        return Err(SendError::PacketCreateMallocFailure)
      }
      if ll::enet_peer_send(self.raw(), channel_id, raw) < 0 {
        return Err(SendError::Failure)
      }
      Ok(())
    }
  } // end send
} // end impl Peer
