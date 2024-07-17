
## Upgrading the hashed pre-compiled PHP version

By default, for PHP-based custom uploaders, MagicCap will only allow the version of PHP with the hash embedded into the application to be executed. This is done to prevent trust based attacks and means that if the version of PHP changes within the app, users will be prompted to update their version of PHP. It is worth noting that PHP is NOT bundled with MagicCap, rather the hash and direct download link for the latest in `WebScaleSoftwareLtd/magiccap-php-binaries` (or [from the official PHP mirror for Windows](https://windows.php.net/download/)).

## Developing MagicCap

To develop MagicCap, you will want to install the following on your computer:

- Node 20.11+
- The Rust toolchain
- XCode Developer Tools (if on macOS)
- Foreman (generally I suggest installing Ruby and doing `gem install foreman` for this, but something foreman compatible will work too)

If this is your first time cloning the repository, you should run `make dev-preinit`. You can then run `make macos-dev` on macOS or `make linux-dev` on Linux.
