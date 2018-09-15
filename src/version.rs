use {std, ll};

#[derive(Clone, Copy, Debug)]
pub struct Version {
  version : ll::ENetVersion
}

pub fn linked_version() -> Version {
  unsafe {
    Version { version: ll::enet_linked_version() }
  }
}

impl Version {
  pub fn create (major : u32, minor : u32, patch : u32) -> Self {
    Version { version: (major << 16) | (minor << 8) | patch }
  }
  pub fn get_major (self) -> u32 {
    (self.version >> 16) & 0xff
  }
  pub fn get_minor (self) -> u32 {
    (self.version >> 8) & 0xff
  }
  pub fn get_patch (self) -> u32 {
    self.version & 0xff
  }
} // end impl Version

impl std::fmt::Display for Version {
  fn fmt (&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}.{}.{}", self.get_major(), self.get_minor(), self.get_patch())
  }
}
