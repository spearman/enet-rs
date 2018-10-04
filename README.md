# `enet`

> Rust interface for the [ENet reliable UDP library](http://enet.bespin.org/)

## Usage

ENet must be initialized before use:

```
let enet = enet::initialize().unwrap();
```

and will return an `enet::Error::Initialize` if initialization failed or ENet
was already initialized.

From an initialized ENet struct, hosts can be created:

```
let enet = enet::initialize().unwrap();
let mut client = enet.client_host_create(
  1,                // only allow 1 outgoing connection (peer)
  Some (57600 / 8), // 56 Kbps downstream bandwidth
  Some (14400 / 8)  // 14 Kbps upstream bandwidth
).unwrap();
let mut server = enet.server_host_create (
  address,  // address to bind the server host to
  32,       // allow up to 32 clients and/or outgoing connections (peers)
  Some (2), // allow up to 2 channels to be used, 0 and 1
  None,     // assume any amount of incoming bandwidth
  None      // assume any amount of outgoing bandwidth
).unwrap();
```

The peer count given when creating a host defines the number of "connections"
to other peers that host may have.

*Client vs. Server*

"Servers" can both accept incoming connections and request outgoing connections,
while a "client" will not listen for incoming connections.

The server host determines the channels which are available for connections. If
a server host only ever accepts connections, then no packets will ever
successfuly send or broadcast if sent with `channelID >= host.channelLimit`,
since the recipient of a connection request (the server) determines the maximum
channel ID.

For a "client" host, a connection with a server can use as many channels as the
server allows, so `host.channelLimit` is not used in this case.


**Peers**

When connecting to a peer with the `host.connect()` method, a `Peer`
representing the connection will be created in the `PeerState::Connecting`
state:
```
let mut peer = client.connect (&address::localhost (12345), 2, 0)
```
where the second argument (`2`) is the number of channels to allocate to the
connection and the third argument (`0`) is an internal `data : u32` that can be
used by the application.

After receipt of a `Connect` event, the peer is ready to use.

*Note*: after receipt of the `Connect` event on the host that originated the
connection request, a call to `flush()` or `service()` is required in order to
*acknowledge* the connection has succeeded in order to generate the
corresponding `Connect` event on the server end.

That connection will now be 'used' until the peer is changed to the
`PeerState::Disconnected` state.

Note that `Host`s can connect *mutually* (host A connected to host B, and host B
connected to host A), or *multiply* (host A connected to host B more than 1
time), and each connection will have its own `Peer` structure in each host A and
B.
