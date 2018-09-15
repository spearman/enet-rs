use {std, ll};
use {packet, peer, Address, EnetDrop, Event, Packet, Peer};

////////////////////////////////////////////////////////////////////////////////
//  structs                                                                   //
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct Host {
  hostdrop : std::rc::Rc <HostDrop>
}

#[derive(Debug, PartialEq)]
pub struct HostDrop {
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
  /// Maximum peer count is 4096
  TooManyPeers (u32),
  ReturnedNull
}

////////////////////////////////////////////////////////////////////////////////
//  impls                                                                     //
////////////////////////////////////////////////////////////////////////////////

impl Host {
  pub fn new (
    address            : Option <Address>,
    peer_count         : u32,
    channel_limit      : Option <usize>,
    incoming_bandwidth : Option <u32>,
    outgoing_bandwidth : Option <u32>,
    enetdrop           : std::sync::Arc <EnetDrop>
  ) -> Result <Self, CreateError> {
    if ll::ENET_PROTOCOL_MAXIMUM_PEER_ID < peer_count {
      return Err (CreateError::TooManyPeers (peer_count))
    }
    let host;
    match address {
      Some (a) => unsafe {
        host = ll::enet_host_create (
          a.raw(),
          peer_count as usize,
          channel_limit.unwrap_or      (0),
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
          peer_count as usize,
          channel_limit.unwrap_or      (0),
          incoming_bandwidth.unwrap_or (0),
          outgoing_bandwidth.unwrap_or (0)
        );
        if host.is_null() {
          return Err (CreateError::ReturnedNull)
        }
      }
    } // end match address
    Ok (Host {
      hostdrop : std::rc::Rc::new (HostDrop {
        raw: host,
        enetdrop
      })
    })
  } // end new

  pub fn broadcast (&mut self, channel_id : u8, packet : Packet) {
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
      ll::enet_host_broadcast (self.raw(), channel_id, raw)
    }
  }

  pub fn connect (&mut self, address : &Address, channel_count : u8, data : u32)
    -> Result <Peer, peer::ConnectError>
  {
    unsafe {
      if (*self.raw()).peerCount <= (*self.raw()).connectedPeers {
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

  #[inline]
  pub unsafe fn raw (&self) -> *mut ll::ENetHost {
    self.hostdrop.raw()
  }

  /// Send any queued messages without dispatching events
  #[inline]
  pub fn flush (&mut self) {
    unsafe { ll::enet_host_flush (self.hostdrop.raw) }
  }

  pub fn service (&mut self, timeout : u32) -> Result <Option <Event>, Error> {
    unsafe {
      let mut event = std::mem::uninitialized::<ll::ENetEvent>();
      if ll::enet_host_service (self.hostdrop.raw, &mut event, timeout) < 0 {
        return Err (Error::ServiceError)
      }
      match event.type_ {
        ll::_ENetEventType_ENET_EVENT_TYPE_NONE       => Ok (None),
        ll::_ENetEventType_ENET_EVENT_TYPE_CONNECT    => {
          Ok (Some (Event::Connect {
            peer: Peer::from_raw (event.peer, self.hostdrop.clone()),
            data: event.data
          }))
        }
        ll::_ENetEventType_ENET_EVENT_TYPE_DISCONNECT => {
          Ok (Some (Event::Disconnect {
            peer: Peer::from_raw (event.peer, self.hostdrop.clone()),
            data: event.data
          }))
        }
        ll::_ENetEventType_ENET_EVENT_TYPE_RECEIVE    => {
          Ok (Some (Event::Receive {
            peer:       Peer::from_raw (event.peer, self.hostdrop.clone()),
            channel_id: event.channelID,
            packet:     packet::PacketRecv::from_raw (event.packet)
          }))
        }
        // TODO: compiler hint ?
        _ => unreachable!()
      }
    }
  } // end service

  #[inline]
  pub fn check_events (&mut self) -> Result <Option <Event>, Error> {
    unsafe {
      let mut event = std::mem::uninitialized::<ll::ENetEvent>();
      if ll::enet_host_check_events (self.hostdrop.raw, &mut event) < 0 {
        return Err (Error::DispatchError)
      }
      match event.type_ {
        ll::_ENetEventType_ENET_EVENT_TYPE_NONE       => Ok (None),
        ll::_ENetEventType_ENET_EVENT_TYPE_CONNECT    => {
          Ok (Some (Event::Connect {
            peer: Peer::from_raw (event.peer, self.hostdrop.clone()),
            data: event.data
          }))
        }
        ll::_ENetEventType_ENET_EVENT_TYPE_DISCONNECT => {
          Ok (Some (Event::Disconnect {
            peer: Peer::from_raw (event.peer, self.hostdrop.clone()),
            data: event.data
          }))
        }
        ll::_ENetEventType_ENET_EVENT_TYPE_RECEIVE    => {
          Ok (Some (Event::Receive {
            peer:       Peer::from_raw (event.peer, self.hostdrop.clone()),
            channel_id: event.channelID,
            packet:     packet::PacketRecv::from_raw (event.packet)
          }))
        }
        // TODO: compiler hint ?
        _ => unreachable!()
      }
    }
  }
} // end impl Host

impl HostDrop {
  #[inline]
  pub unsafe fn raw (&self) -> *mut ll::ENetHost {
    self.raw
  }
}
impl Drop for HostDrop {
  #[inline]
  fn drop (&mut self) {
    unsafe { ll::enet_host_destroy (self.raw) }
  }
}
