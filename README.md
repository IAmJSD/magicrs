
## Developing MagicCap

To develop MagicCap, you will want to install the following on your computer:

- Node 20.11+
- The yarn package manager
- The Rust toolchain
- XCode Developer Tools (if on macOS)
- Foreman (generally I suggest installing Ruby and doing `gem install foreman` for this, but something foreman compatible will work too)

If this is your first time cloning the repository, you should run `make dev-preinit`. You can then run `make macos-dev` on macOS or `make linux-dev` on Linux.
