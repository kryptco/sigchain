# Sigchain | Krypton for Teams
Implements the `signed hash chain`, or `sigchain`, protocol for Krypton. 
Learn more here: [Overview of the Krypton Sigchain Protocol](https://www.krypt.co/docs/sigchain/team-sigchain.html).

## What is Krypton?
<a href="https://krypt.co"><img src="https://krypt.co/static/dist/img/krypton_core_logo.svg" width="120"/> </a>

__Krypton__ generates and stores an SSH key pair on a mobile phone. The
Krypton app is paired with one or more workstations by scanning a QR code
presented in the terminal. When using SSH from a paired workstation, the
workstation requests a private key signature from the phone. The user then
receives a notification and chooses whether to allow the SSH login.

For more information, check out [krypt.co](https://krypt.co).

# Dependencies
- Rust 1.24+ 
- libsodium
- emscripten

# Build
```shell
$ git clone git@github.com:kryptco/sigchain --recursive
$ cd sigchain
$ cargo build
```

# Components
- `sigchain_client`: Implements all the various types of Sigchain clients. Examples of clients:
    - `DelegatedNetworkClient`: a client that has delegated access to a keypair, i.e. it sends Krypton Requests to perform team operation
    - `NetworkClient`: A client that hits a network Sigchain server
    - `TestClient`: A client with a mocked Sigchain server.
- `libsigchain`: Creates a `C` interface for using a `DelegatedNetworkClient`. Used by [kr](https://github.com/kryptco/kr) for `kr team` [commands](https://www.krypt.co/docs/teams/command-line.html)
- `dashboard_middleware`: The local webserver back-end used by the `dashboard_yew` frontend.
- `dashboard_yew`: Implements a [Web UI](https://www.krypt.co/docs/teams/dashboard.html) in WebAssembly to run a `DelegatedNetworkClient` in a web UI.
- `sigchain_core`: Shared components used by all the above.

# Security Disclosure Policy
__krypt.co__ follows a 7-day disclosure policy. If you find a security flaw,
please send it to `disclose@krypt.co` encrypted to the PGP key with fingerprint
`B873685251A928262210E094A70D71BE0646732C` ([Full PGP Key](https://www.krypt.co/docs/security/disclosure-policy.html)). 
We ask that you delay publication of the flaw until we have published a fix, or seven days have
passed.

# License
We are currently working on a new license for Krypton. For now, the code
is released under All Rights Reserved.
