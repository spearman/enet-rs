use {ll, std};

#[derive(Clone, Debug)]
pub struct Address {
  raw : ll::ENetAddress
}

//
//  AddressError
//

#[derive(Debug)]
pub enum AddressError {
  HostNameResolveFailure(String),
  CStringNulError(std::ffi::NulError)
}

////////////////////////////////////////////////////////////////////////////////
//  impls                                                                     //
////////////////////////////////////////////////////////////////////////////////

impl Address {
  /// Default localhost
  pub fn new(port : u16) -> Address {
    let host = ll::ENET_HOST_ANY;
    let raw = ll::ENetAddress { host, port };
    Address { raw }
  }
  pub fn with_hostname(
    hostname : &str,
    port : u16
  ) -> Result<Address, AddressError> {
    let cname = try!(std::ffi::CString::new(hostname));
    unsafe {
      let mut address = ll::ENetAddress { host : 0, port : 0 };
      if ll::enet_address_set_host(&mut address, cname.as_ptr()) < 0 {
        return Err(AddressError::HostNameResolveFailure(hostname.to_string()))
      }
      Ok(Address {
        raw : ll::ENetAddress { port, ..address }
      })
    }
  }

  #[inline]
  pub fn raw(&self) -> *const ll::ENetAddress {
    &self.raw
  }
  #[inline]
  pub fn raw_mut(&mut self) -> *mut ll::ENetAddress {
    &mut self.raw
  }
  #[inline]
  pub fn host(self) -> u32 {
    self.raw.host
  }
  #[inline]
  pub fn port(self) -> u16 {
    self.raw.port
  }
} // end impl Address

impl From<std::ffi::NulError> for AddressError {
  fn from(err : std::ffi::NulError) -> AddressError {
    AddressError::CStringNulError(err)
  }
}
