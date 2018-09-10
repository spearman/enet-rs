use {ll, std};
use {packet, Address, EnetDrop, Event, Packet, Peer};

////////////////////////////////////////////////////////////////////////////////
//  structs                                                                   //
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct Host {
  hostdrop : std::rc::Rc<HostDrop>
}

#[derive(Debug)]
pub struct HostServiceError;

#[derive(Debug)]
pub struct HostDrop {
  raw :      *mut ll::ENetHost,
  enetdrop : std::sync::Arc<EnetDrop>
}

////////////////////////////////////////////////////////////////////////////////
//  enums                                                                     //
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum HostConnectError {
  NoPeersAvailable,
  /// failure due to internal malloc failure of channel allocation
  Failure
}

#[derive(Clone, Debug)]
pub enum HostCreateError {
  TooManyPeers(u32),
  ReturnedNull
}

////////////////////////////////////////////////////////////////////////////////
//  impls                                                                     //
////////////////////////////////////////////////////////////////////////////////

impl Host {
  pub fn new(
    address : Option<Address>,
    peer_count : u32,
    channel_limit : Option<usize>,
    incoming_bandwidth : Option<u32>,
    outgoing_bandwidth : Option<u32>,
    enetdrop : std::sync::Arc<EnetDrop>
  ) -> Result<Self, HostCreateError> {
    if ll::ENET_PROTOCOL_MAXIMUM_PEER_ID < peer_count {
      return Err(HostCreateError::TooManyPeers(peer_count))
    }
    let host;
    match address {
      Some(a) => unsafe {
        host = ll::enet_host_create(
          a.raw(),
          peer_count as usize,
          channel_limit.unwrap_or(0),
          incoming_bandwidth.unwrap_or(0),
          outgoing_bandwidth.unwrap_or(0)
        );
        if host.is_null() {
          return Err(HostCreateError::ReturnedNull)
        }
      },
      None => unsafe {
        host = ll::enet_host_create(
          std::ptr::null(),
          peer_count as usize,
          channel_limit.unwrap_or(0),
          incoming_bandwidth.unwrap_or(0),
          outgoing_bandwidth.unwrap_or(0)
        );
        if host.is_null() {
          return Err(HostCreateError::ReturnedNull)
        }
      }
    } // end match address
    Ok(Host {
      hostdrop : std::rc::Rc::new(HostDrop {
        raw : host,
        enetdrop
      })
    })
  } // end new

  pub fn broadcast(&mut self, channel_id : u8, packet : Packet) {
    unsafe {
      let raw;
      match packet {
        Packet::Allocate { bytes, flags } => {
          raw = ll::enet_packet_create(
            bytes.as_ptr() as (*const std::os::raw::c_void),
            bytes.len(),
            flags.bits()
          );
        }
        Packet::NoAllocate { bytes, flags } => {
          raw = ll::enet_packet_create(
            bytes.as_ptr() as (*const std::os::raw::c_void),
            bytes.len(),
            flags.bits() | ll::_ENetPacketFlag_ENET_PACKET_FLAG_NO_ALLOCATE
          );
        }
      }
      ll::enet_host_broadcast(self.raw(), channel_id, raw)
    }
  }

  pub fn connect(
    &mut self,
    address : &Address,
    channel_count : u8,
    data : u32
  ) -> Result<Peer, HostConnectError> {
    unsafe {
      if (*self.raw()).peerCount <= (*self.raw()).connectedPeers {
        return Err(HostConnectError::NoPeersAvailable)
      }
      let peer = ll::enet_host_connect(
        self.raw(),
        address.raw(),
        channel_count as usize,
        data
      );
      if peer.is_null() {
        return Err(HostConnectError::Failure)
      }
      Ok(Peer::from_raw(peer, self.hostdrop.clone()))
    }
  }

  #[inline]
  pub unsafe fn raw(&self) -> *mut ll::ENetHost {
    self.hostdrop.raw()
  }

  pub fn service(
    &mut self,
    timeout : u32
  ) -> Result<Option<Event>, HostServiceError> {
    unsafe {
      let mut event = std::mem::uninitialized::<ll::ENetEvent>();
      if ll::enet_host_service(self.hostdrop.raw, &mut event, timeout) < 0 {
        return Err(HostServiceError)
      }
      match event.type_ {
        ll::_ENetEventType_ENET_EVENT_TYPE_NONE => Ok(None),
        ll::_ENetEventType_ENET_EVENT_TYPE_CONNECT => {
          Ok(Some(Event::Connect {
            peer : Peer::from_raw(event.peer, self.hostdrop.clone()),
            data : event.data
          }))
        }
        ll::_ENetEventType_ENET_EVENT_TYPE_DISCONNECT => {
          Ok(Some(Event::Disconnect {
            peer : Peer::from_raw(event.peer, self.hostdrop.clone()),
            data : event.data
          }))
        }
        ll::_ENetEventType_ENET_EVENT_TYPE_RECEIVE => {
          Ok(Some(Event::Receive {
            peer :       Peer::from_raw(event.peer, self.hostdrop.clone()),
            channel_id : event.channelID,
            packet :     packet::PacketRecv::from_raw(event.packet)
          }))
        }
        // TODO: compiler hint ?
        _ => unreachable!()
      }
    }
  } // end service
} // end impl Host

impl HostDrop {
  #[inline]
  pub unsafe fn raw(&self) -> *mut ll::ENetHost {
    self.raw
  }
}
impl Drop for HostDrop {
  #[inline]
  fn drop(&mut self) {
    unsafe { ll::enet_host_destroy(self.raw) }
  }
}
