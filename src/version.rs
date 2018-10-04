use {std, ll};

#[derive(Clone, Copy, Debug)]
pub struct Version {
  version : ll::ENetVersion
}

impl Version {
  #[inline]
  pub fn from_ll (version : ll::ENetVersion) -> Self {
    Version { version }
  }
  #[inline]
  pub fn create (major : u32, minor : u32, patch : u32) -> Self {
    Version { version: (major << 16) | (minor << 8) | patch }
  }
  #[inline]
  pub fn get_major (self) -> u32 {
    (self.version >> 16) & 0xff
  }
  #[inline]
  pub fn get_minor (self) -> u32 {
    (self.version >> 8) & 0xff
  }
  #[inline]
  pub fn get_patch (self) -> u32 {
    self.version & 0xff
  }
}

impl std::fmt::Display for Version {
  fn fmt (&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}.{}.{}", self.get_major(), self.get_minor(), self.get_patch())
  }
}
