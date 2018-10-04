use {packet, Peer};

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
