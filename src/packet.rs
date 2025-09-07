use {std, ll};

////////////////////////////////////////////////////////////////////////////////
//  structs                                                                   //
////////////////////////////////////////////////////////////////////////////////

/// Outgoing packet for peer `send()`
#[derive(Clone, Copy, Debug)]
pub enum Packet <'a> {
  Allocate {
    bytes : &'a [u8],
    flags : Flags
  },
  NoAllocate {
    bytes : &'static [u8],
    flags : Flags
  }
}

/// Received packet
#[derive(Debug)]
pub struct PacketRecv {
  raw : *mut ll::ENetPacket
}

bitflags! {
  /// Flags for outgoing packets.
  ///
  /// `Flags::empty()` indicates unreliable, sequenced delivery.
  #[derive(Clone, Copy, Debug)]
  pub struct Flags : u32 {
    /// Reliable, sequenced delivery
    #[allow(clippy::unnecessary_cast)] // on windows ll flags are i32
    const RELIABLE    = ll::_ENetPacketFlag_ENET_PACKET_FLAG_RELIABLE as u32;
    /// Unsequenced delivery
    #[allow(clippy::unnecessary_cast)] // on windows ll flags are i32
    const UNSEQUENCED = ll::_ENetPacketFlag_ENET_PACKET_FLAG_UNSEQUENCED as u32;
    /// Packet will be fragmented if it exceeds the MTU
    #[allow(clippy::unnecessary_cast)] // on windows ll flags are i32
    const UNRELIABLE_FRAGMENT = ll::_ENetPacketFlag_ENET_PACKET_FLAG_UNRELIABLE_FRAGMENT
      as u32;
    /// Packet will not allocate data and user must supply it instead
    #[allow(clippy::unnecessary_cast)] // on windows ll flags are i32
    const NO_ALLOCATE = ll::_ENetPacketFlag_ENET_PACKET_FLAG_NO_ALLOCATE as u32;
    // TODO: choose to expose the packet sent flag?
    // Whether the packet has been sent from all queues it has been entered into.
    //#[allow(clippy::unnecessary_cast)] // on windows ll flags are i32
    //const SENT        = ll::_ENetPacketFlag_ENET_PACKET_FLAG_SENT as u32;
  }
}

////////////////////////////////////////////////////////////////////////////////
//  impls                                                                     //
////////////////////////////////////////////////////////////////////////////////

impl PacketRecv {
  /// # Safety
  ///
  /// Unsafe: raw pointer could be invalid.
  #[inline]
  pub const unsafe fn from_raw (raw : *mut ll::ENetPacket) -> Self {
    PacketRecv { raw }
  }

  #[inline]
  pub fn flags (&self) -> Flags {
    unsafe { Flags::from_bits ((*self.raw).flags).unwrap() }
  }

  #[inline]
  pub fn data_length (&self) -> usize {
    unsafe { (*self.raw).dataLength }
  }

  #[inline]
  pub fn data (&self) -> &[u8] {
    unsafe {
      let len = (*self.raw).dataLength;
      std::slice::from_raw_parts ((*self.raw).data, len)
    }
  }

  // TODO: set_packet_free_callback
}
impl Drop for PacketRecv {
  #[inline]
  fn drop (&mut self) {
    unsafe { ll::enet_packet_destroy (self.raw) }
  }
}
