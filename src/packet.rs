use {std, ll};

////////////////////////////////////////////////////////////////////////////////
//  structs                                                                   //
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub enum Packet<'a> {
  Allocate {
    bytes : &'a [u8],
    flags : Flags
  },
  NoAllocate {
    bytes : &'static [u8],
    flags : Flags
  }
}

#[derive(Debug)]
pub struct PacketRecv {
  raw : *mut ll::ENetPacket
}

bitflags! {
  pub struct Flags : u32 {
    const RELIABLE    = ll::_ENetPacketFlag_ENET_PACKET_FLAG_RELIABLE;
    const UNSEQUENCED = ll::_ENetPacketFlag_ENET_PACKET_FLAG_UNSEQUENCED;
    const UNRELIABLE_FRAGMENT =
      ll::_ENetPacketFlag_ENET_PACKET_FLAG_UNRELIABLE_FRAGMENT;
    const SENT        = ll::_ENetPacketFlag_ENET_PACKET_FLAG_SENT;
  }
}

////////////////////////////////////////////////////////////////////////////////
//  impls                                                                     //
////////////////////////////////////////////////////////////////////////////////

impl PacketRecv {
  #[inline]
  pub unsafe fn from_raw (raw : *mut ll::ENetPacket) -> Self {
    PacketRecv { raw }
  }

  #[inline]
  pub fn flags (&self) -> Flags {
    unsafe {
      Flags::from_bits ((*self.raw).flags).unwrap()
    }
  }

  #[inline]
  pub fn data (&self) -> &[u8] {
    unsafe {
      let len = (*self.raw).dataLength;
      std::slice::from_raw_parts ((*self.raw).data, len)
    }
  }
}
impl Drop for PacketRecv {
  #[inline]
  fn drop (&mut self) {
    unsafe { ll::enet_packet_destroy (self.raw) }
  }
}
