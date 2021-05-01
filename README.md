# DERO Guard

> Decentralized VPN service over DERO

Building this project requires a working [Rust](https://rustup.rs/) (stable) toolchain.

## Runtime dependencies

- Linux
- DERO Daemon
- DERO Wallet CLI (with `--rpc-server`)
- Wireguard
- Ipoute2
- iptables

Proper IPv4 forwarding setup must be done server-side, see
[Wireguard Arch wiki page](https://wiki.archlinux.org/index.php/WireGuard)

## Setting-up a server

```bash
$ cargo build --bin dero_guard_server
$ sudo target/debug/dero_guard_server SERVER_API_ADDRESS
```

## Setting-up a client

```bash
$ cargo build --bin dero_guard_client
$ sudo target/debug/dero_guard_client SERVER_DERO_ADDRESS
```

## How does it work

To connect to a VPN server, the client will send a transaction to the server wallet with its public key and DEROs to
pay for the bandwidth usage. The server will answer with another transaction, with its public key, Wireguard port, and
the local IP address attributed to the client. The client will then connect to the server using Wireguard and use it as
a VPN.

The DERO amount sent with the transaction will define how much bandwidth the client can use. If it uses all the bandwidth
it paid for, the connection will be closed by the server. Each server can define its own price.

In the future, a GUI will be added to the client which will be able to see a list of available servers with their
location and bandwidth price. The client also only works on Linux now but will be adapted to work on both macOS and
Windows. The server will remain available on Linux only.

Bandwidth monitoring isn't integrated for now.

## Result

![Example](https://cdn.discordapp.com/attachments/226418730542956544/838119630634876998/unknown.png)
![Example 2](https://cdn.discordapp.com/attachments/226418730542956544/838119923954876416/unknown.png)