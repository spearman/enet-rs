use {std, ll};

#[derive(Clone)]
pub struct Address {
  address : ll::ENetAddress
}

//
//  AddressError
//

#[derive(Debug)]
pub enum AddressError {
  HostNameResolveFailure (String),
  CStringNulError        (std::ffi::NulError)
}

////////////////////////////////////////////////////////////////////////////////
//  impls                                                                     //
////////////////////////////////////////////////////////////////////////////////

impl Address {
  pub (crate) fn from_ll (address : ll::ENetAddress) -> Self {
    Address { address }
  }
  /// 127.0.0.1
  #[inline]
  pub fn localhost (port : u16) -> Address {
    Address::with_hostname ("127.0.0.1", port).unwrap()
  }
  /// Creates an address with `ENET_HOST_ANY` (0.0.0.0)
  pub fn any (port : u16) -> Address {
    let host     = ll::ENET_HOST_ANY;
    let address  = ll::ENetAddress { host, port };
    Address { address }
  }
  pub fn with_hostname (
    hostname : &str,
    port     : u16
  ) -> Result<Address, AddressError> {
    let cname = std::ffi::CString::new(hostname)?;
    unsafe {
      let address = {
        let mut address = ll::ENetAddress { host: 0, port: 0 };
        if ll::enet_address_set_host(&mut address, cname.as_ptr()) < 0 {
          return Err(AddressError::HostNameResolveFailure(hostname.to_string()))
        }
        ll::ENetAddress { port, .. address }
      };
      Ok(Address { address })
    }
  }
  /// # Safety
  ///
  /// Unsafe: returns a raw pointer.
  #[inline]
  pub unsafe fn raw (&self) -> *const ll::ENetAddress {
    &self.address
  }
  /// # Safety
  ///
  /// Unsafe: returns a raw pointer.
  #[inline]
  pub unsafe fn raw_mut (&mut self) -> *mut ll::ENetAddress {
    &mut self.address
  }
  #[inline]
  pub fn host (self) -> u32 {
    self.address.host
  }
  #[inline]
  pub fn host_bytes (self) -> [u8; 4] {
    self.address.host.to_le_bytes()
  }
  #[inline]
  pub fn port (self) -> u16 {
    self.address.port
  }
} // end impl Address
impl Default for Address {
  /// 127.0.0.1:80
  fn default() -> Self {
    Address::localhost (80)
  }
}
impl std::fmt::Debug for Address {
  fn fmt (&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
    let host = self.address.host.to_le_bytes();
    let port = self.address.port;
    write!(f, "Address {{ host: {}.{}.{}.{}, port: {} }}",
      host[0], host[1], host[2], host[3], port)
  }
}

impl From <std::ffi::NulError> for AddressError {
  fn from (err : std::ffi::NulError) -> AddressError {
    AddressError::CStringNulError (err)
  }
}
