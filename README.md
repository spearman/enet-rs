# `enet`

> Rust interface for the [ENet reliable UDP library](http://enet.bespin.org/)

## Usage

ENet must be initialized before use:
```
let enet = enet::initialize().unwrap();
```
and will return `enet::Error::Initialize` if initialization failed or ENet
was already initialized.

From an initialized ENet struct, hosts can be created which will keep the enet
context alive as long as references to them or their peers exist:
```
let enet = enet::initialize().unwrap();
let mut client = enet.client_host_create(
  1,        // only allow 1 outgoing connection (peer)
  None,     // allow any amount of in-flight reliable downstream bandwidth
  None      // allow any amount of in-flight reliable upstream bandwidth
).unwrap();
let mut server = enet.server_host_create (
  address,  // address to bind the server host to
  32,       // allow up to 32 clients and/or outgoing connections (peers)
  Some (2), // allow up to 2 channels to be used, 0 and 1
  None,     // allow any amount of in-flight reliable downstream bandwidth
  None      // allow any amount of in-flight reliable upstream bandwidth
).unwrap();
```

A "client" is just a server that is not intended to listen for incoming
connections. The number of channels used is always determined by the server-end
of a connection.

## Building

### Windows 10

In addition to installing the Rust toolchain and MSVC, the `enet-sys` dependency
on Windows 10 requires Clang and CMake to be installed.

**Install Clang**

Install the latest Clang release from
<https://github.com/llvm/llvm-project/releases> by downloading and running
`LLVM-20.1.4-win64.exe`. In the installer, the option to add LLVM to the PATH
for all users.

**Install CMake**

Install CMake from <https://cmake.org/download/> by downloading and running
`cmake-4.0.1-windows-x86_64.msi`. In the installer ensure that the option to add
CMake to the PATH variable is selected (it should be by default).

**Build**

`enet-rs` should now build successfully with `cargo build` from the command
prompt or PowerShell.

To build from an MSYS2 shell (URCT64 or MINGW64 environments), the Clang and
CMake bin paths will need to be added to `.bash_profile`, e.g.:
```
PATH="/c/Program Files/LLVM/bin:/c/Program Files/CMake/bin:$PATH"
```
