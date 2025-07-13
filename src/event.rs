use ll;
use crate::{host, packet, Peer};

use std::rc::Rc;

/// Event structure returned by `host.service()` or `host.check_events()`
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

impl Event {
  pub (crate) fn from_ll (event : ll::ENetEvent, hostdrop : Rc <host::HostDrop>)
    -> Option <Self>
  {
    unsafe {
      match event.type_ {
        ll::_ENetEventType_ENET_EVENT_TYPE_NONE       => None,
        ll::_ENetEventType_ENET_EVENT_TYPE_CONNECT    => {
          Some (Event::Connect {
            peer: Peer::from_raw (event.peer, hostdrop),
            data: event.data
          })
        }
        ll::_ENetEventType_ENET_EVENT_TYPE_DISCONNECT => {
          Some (Event::Disconnect {
            peer: Peer::from_raw (event.peer, hostdrop),
            data: event.data
          })
        }
        ll::_ENetEventType_ENET_EVENT_TYPE_RECEIVE    => {
          Some (Event::Receive {
            peer:       Peer::from_raw (event.peer, hostdrop),
            channel_id: event.channelID,
            packet:     packet::PacketRecv::from_raw (event.packet)
          })
        }
        // TODO: compiler hint ?
        _ => unreachable!()
      }
    }
  }
}
