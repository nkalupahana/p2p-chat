# P2P Messaging Client (using NAT Hole Punching)

![demo.png](demo.png)

This project is a basic P2P messaging client that utilizes NAT hole punching. It has two parts:

- `server/`: used for initial connection and to exchange NAT IP & port.
- `client/`: used for actually sending and receiving messages. Connects to server, then uses information to make P2P connection.


> NOTE: NAT hole punching doesn't work on all networks. [Before you use this, ensure that you have a 
permissive NAT that allows for hole punching.](https://clients.dh2i.com/NatTest/) Of note, vuNet does not support NAT hole punching.

Two clients that use the same "keyword" can send messages between each other, as you'd expect from a standard messaging client.

## Installation & Running (Server)

In order to run the server, you'll need [Rust and Cargo installed](https://www.rust-lang.org/tools/install). To run, go into the server folder and run `cargo run --release`.

## Installation & Running (Client)

In order to run the client, you'll need [Rust and Cargo installed](https://www.rust-lang.org/tools/install).

For GUI support: on macOS and Windows, nothing extra is needed. However, on Linux, you'll need to run this:
```
# Distros that use apt
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev

# Fedora/Derivatives
dnf install clang clang-devel clang-tools-extra speech-dispatcher-devel libxkbcommon-devel pkg-config openssl-devel
```
If you can't install these, you can just run in console mode, as shown below.

To run, go into the client folder and run `cargo run --release`. If you don't have a display, console mode will automatically be launched; otherwise, a GUI will launch once a connection is established. If you'd like to use console mode directly, run `cargo build --release`, go into `target/release`, and run `./client --console`. 

## Project Details

I chose this as my project because it seemed interesting, and I'd heard about it from friends who'd had issues with P2P because
of Vanderbilt's NAT. I thus wanted to learn more about how NATs work and the problems with P2P.

During implementation, I had issues with:
- Multithreading -- although with Rust, once I learned how to do it, it was super easy. In Rust, you can use a
    reference counter, and then clone references and move them into each thread you want to use the reference in. Simple as that.
- The theory of NAT hole punching -- this stuff isn't super well documented. I found a [good flow chart
    on Wikipedia](https://en.wikipedia.org/wiki/UDP_hole_punching#Flow) that I used for actual implementation.
- Setting up Terraform. Declarative deployments are hard. I just had to spend a lot of time with documentation to figure out
    how to do everything, that's all. It turns out that when GCP runs startup scripts, a lot of environment variables (like `$HOME`)
    aren't set, for some reason? So it was just a lot of debugging.

