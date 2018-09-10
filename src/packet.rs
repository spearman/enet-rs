use ll;

////////////////////////////////////////////////////////////////////////////////
//  structs                                                                   //
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub enum Packet<'a> {
  Allocate {
    bytes : &'a [u8],
    flags : PacketFlags
  },
  NoAllocate {
    bytes : &'static [u8],
    flags : PacketFlags
  }
}

#[derive(Debug)]
pub struct PacketRecv {
  raw : *mut ll::ENetPacket
}

bitflags! {
  pub struct PacketFlags : u32 {
    const RELIABLE    = ll::_ENetPacketFlag_ENET_PACKET_FLAG_RELIABLE;
    const UNSEQUENCED = ll::_ENetPacketFlag_ENET_PACKET_FLAG_UNSEQUENCED;
    const UNRELIABLE_FRAGMENT =
      ll::_ENetPacketFlag_ENET_PACKET_FLAG_UNRELIABLE_FRAGMENT;
    const SENT = ll::_ENetPacketFlag_ENET_PACKET_FLAG_SENT;
  }
}

////////////////////////////////////////////////////////////////////////////////
//  impls                                                                     //
////////////////////////////////////////////////////////////////////////////////

impl PacketRecv {
  #[inline]
  pub unsafe fn from_raw(raw : *mut ll::ENetPacket) -> Self {
    PacketRecv { raw }
  }
}

impl Drop for PacketRecv {
  #[inline]
  fn drop(&mut self) {
    unsafe { ll::enet_packet_destroy(self.raw) }
  }
}
