[![Build status on GitLab CI][gitlab-ci-master-badge]][gitlab-ci-link]
[![Newest release on crates.io][crate-version-badge]][crate-link]
[![Project license][crate-license-badge]](LICENSE)

[crate-license-badge]: https://img.shields.io/crates/l/ffsend.svg
[crate-link]: https://crates.io/crates/ffsend
[crate-version-badge]: https://img.shields.io/crates/v/ffsend.svg
[gitlab-ci-link]: https://gitlab.com/timvisee/ffsend/pipelines
[gitlab-ci-master-badge]: https://gitlab.com/timvisee/ffsend/badges/master/pipeline.svg

*Notice: the default Send host is provided by [@timvisee][timvisee]
([info](https://gitlab.com/timvisee/ffsend/-/issues/111)).
Please consider to [donate] and help keep it running.*

# ffsend

> Easily and securely share files from the command line.
> A [Send][send] client.

Easily and securely share files and directories from the command line through a
safe, private and encrypted link using a single simple command.
Files are shared using the [Send][send] service and may be up
to 1GB. Others are able to download these files with this tool, or through
their web browser.

[![ffsend usage demo][usage-demo-svg]][usage-demo-asciinema]  
_No demo visible here? View it on [asciinema][usage-demo-asciinema]._

All files are always encrypted on the client, and secrets are never shared with
the remote host. An optional password may be specified, and a default file
lifetime of 1 (up to 20) download or 24 hours is enforced to ensure your stuff
does not remain online forever.
This provides a secure platform to share your files.
Find out more about security [here](#security).

- [Features](#features)
- [Usage](#usage)
- [Requirements](#requirements)
- [Install](#install) ([Linux](#linux-all-distributions), [macOS](#macos), [Windows](#windows), [FreeBSD](#freebsd), [Android](#android), [_Other OS/arch_](#other-os-or-architecture))
- [Build](#build)
- [Configuration and environment](#configuration-and-environment)
- [Security](#security)
- [Help](#help)
- [Special thanks](#special-thanks)
- [License](#license)

The public [Send][send] service that is used as default host is provided by
[@timvisee][timvisee] ([info](https://gitlab.com/timvisee/ffsend/-/issues/111)).  
This application is not affiliated with [Firefox][firefox] or
[Mozilla][mozilla] in any way.

_Note: this tool is currently in beta, as some extra desired features are yet to be implemented_

## Features
- Fully featured and friendly command line tool
- Upload and download files and directories securely, always encrypted on the client
- Additional password protection, generation and configurable download limits
- File and directory archiving and extraction
- Built-in share URL shortener and QR code generator
- Supports Send v3 (current) and v2
- History tracking your files for easy management
- Ability to use your own Send hosts
- Inspect or delete shared files
- Accurate error reporting
- Streaming encryption and uploading/downloading, very low memory footprint
- Intended for use in [scripts](#scriptability) without interaction

For a list of upcoming features and ideas, take a look at the
current [open issues](https://gitlab.com/timvisee/ffsend/issues) over on GitLab.

## Usage
Easily upload and download:

```bash
# Simple upload
$ ffsend upload my-file.txt
https://send.vis.ee/#sample-share-url

# Advanced upload
# - Specify a download limit of 1
# - Specify upload expiry time of 5 minutes
# - Enter a password to encrypt the file
# - Archive the file before uploading
# - Copy the shareable link to your clipboard
# - Open the shareable link in your browser
$ ffsend upload --downloads 1 --expiry-time 5m --password --archive --copy --open my-file.txt
Password: ******
https://send.vis.ee/#sample-share-url

# Upload to your own host
$ ffsend u -h https://example.com/ my-file.txt
https://example.com/#sample-share-url

# Simple download
$ ffsend download https://send.vis.ee/#sample-share-url
```

Inspect remote files:

```bash
# Check if a file exists
$ ffsend exists https://send.vis.ee/#sample-share-url
Exists: yes

# Fetch remote file info
$ ffsend info https://send.vis.ee/#sample-share-url
ID:         b087066715
Name:       my-file.txt
Size:       12 KiB
MIME:       text/plain
Downloads:  0 of 10
Expiry:     18h2m (64928s)
```

Other commands include:

```bash
# View your file history
$ ffsend history
#  LINK                                        EXPIRE
1  https://send.vis.ee/#sample-share-url  23h57m
2  https://send.vis.ee/#other-sample-url  17h38m
3  https://example.com/#sample-share-url       37m30s

# Change the password after uploading
$ ffsend password https://send.vis.ee/#sample-share-url
Password: ******

# Delete a file
$ ffsend delete https://send.vis.ee/#sample-share-url
```

Use the `--help` flag, `help` subcommand, or see the [help](#help) section for
all available subcommands.

## Requirements
- Linux, macOS, Windows, FreeBSD, Android (other BSDs might work)
- A terminal :sunglasses:
- Internet connection
- Linux:
  - OpenSSL & CA certificates:
    - Ubuntu, Debian and derivatives: `apt install openssl ca-certificates`
  - Optional: `xclip` or `xsel` for clipboard support
    - Ubuntu, Debian and derivatives: `apt install xclip`
    - CentOS/Red Hat/openSUSE/Fedora: `yum install xclip`
    - Arch: `pacman -S xclip`
- Windows specific:
  - Optional OpenSSL with `crypto-openssl` feature: [» Installer][openssl-windows-installer] (`v1.1.0j` or above)
- macOS specific:
  - Optional OpenSSL with `crypto-openssl` feature: `brew install openssl@1.1`
- FreeBSD specific:
  - OpenSSL: `pkg install openssl`
  - CA certificates: `pkg install ca_root_nss`
  - Optional `xclip` & `xsel` for clipboard support: `pkg install xclip xsel-conrad`
- Android specific:
  - Termux: [» Termux][termux]

## Install
Because `ffsend` is still in early stages, only limited installation options are
available right now. Feel free to contribute additional packages.

Make sure you meet and install the [requirements](#requirements).

See the operating system specific instructions below:
- [Linux](#linux-all-distributions)
- [macOS](#macos)
- [Windows](#windows)
- [FreeBSD](#freebsd)
- [Android](#android)
- [_Other OS or architecture_](#other-os-or-architecture)

### Linux (all distributions)
Using the [snap](#linux-snap-package) package is recommended if supported.  
Alternatively you may install it manually using the
[prebuilt binaries](#linux-prebuilt-binaries).

Only 64-bit (`x86_64`) packages and binaries are provided.
For other architectures and configurations you may [compile from source](#build).

More packages options will be coming soon.

#### Linux: snap package
_Note: The `ffsend` `snap` package is isolated, and can only access files in
your home directory. Choose a different installation option if you don't want
this limitation._

_Note: due to how `snap` is configured by default, you won't be able to use the
package from some contexts such as through SSH without manual modifications. If
you're experiencing problems, please refer to a different installation method
such as the [prebuilt binaries](#linux-prebuilt-binaries), or open an issue._
_Note: if you want to read/write to a flash drive run `snap connect ffsend:removable-media`

[» `ffsend`][snapcraft-ffsend]

```bash
snap install ffsend
ffsend --help
```

#### Linux: Arch AUR packages
[» `ffsend-bin`][aur-ffsend-bin] (precompiled binary, latest release, recommended)  
[» `ffsend`][aur-ffsend] (compiles from source, latest release)  
[» `ffsend-git`][aur-ffsend-git] (compiles from source, latest `master` commit)

```bash
yay -S ffsend
# or
aurto add ffsend-bin
sudo pacman -S ffsend-bin
# or using any other AUR helper

ffsend --help
```

#### Linux: Nix package
_Note: The Nix package is currently not automatically updated, and might be
slightly outdated._

[» ffsend][nix-ffsend]

```bash
nix-channel --update
nix-env --install ffsend

ffsend --help
```

#### Linux: Fedora package
_Note: The Fedora package is maintained by contributors, and might be
slightly outdated._

[» ffsend][fedora-ffsend]

```bash
sudo dnf install ffsend

ffsend --help
```

#### Linux: Alpine package
_Note: The Alpine package is maintained by contributors, it might be outdated.
Choose a different installation method if an important update is missing._

[» ffsend][alpine-ffsend]

```bash
apk add ffsend --repository=http://dl-cdn.alpinelinux.org/alpine/edge/testing

ffsend --help
```

#### Linux: Prebuilt binaries
Check out the [latest release][github-latest-release] assets for Linux binaries.  
Use the `ffsend-v*-linux-x64-static` binary, to minimize the chance for issues.
If it isn't available yet, you may use an artifact from a
[previous version][github-releases] instead, until it is available.

Make sure you meet and install the [requirements](#requirements) before you
continue.

You must make the binary executable, and may want to move it into `/usr/bin` to
make it easily executable:

```bash
# Rename binary to ffsend
mv ./ffsend-* ./ffsend

# Mark binary as executable
chmod a+x ./ffsend

# Move binary into path, to make it easily usable
sudo mv ./ffsend /usr/local/bin/

ffsend --help
```

### macOS
Using the [`homebrew` package](#macos-homebrew-package) is recommended.  
Alternatively you may install it via [MacPorts](#macos-macports), or manually using the
[prebuilt binaries](#macos-prebuilt-binaries).

#### macOS: homebrew package
Make sure you've [`homebrew`][homebrew] installed, and run:

```bash
brew install ffsend
ffsend --help
```

#### macOS: MacPorts
_Note: ffsend in MacPorts is currently not automatically updated, and might be
slightly outdated._

Once you have [MacPorts](https://www.macports.org) installed, you can run:

```bash
sudo port selfupdate
sudo port install ffsend
```

#### macOS: Nix package
_Note: The Nix package is currently not automatically updated, and might be
slightly outdated._

```bash
nix-channel --update
nix-env --install ffsend

ffsend --help
```

#### macOS: Prebuilt binaries
Check out the [latest release][github-latest-release] assets for a macOS binary.
If it isn't available yet, you may use an artifact from a
[previous version][github-releases] instead, until it is available.

Then, mark the downloaded binary as an executable.
You then may want to move it into `/usr/local/bin/` to make the `ffsend` command
globally available:

```bash
# Rename file to ffsend
mv ./ffsend-* ./ffsend

# Mark binary as executable
chmod a+x ./ffsend

# Move binary into path, to make it easily usable
sudo mv ./ffsend /usr/local/bin/

ffsend
```

### Windows
Using the [`scoop` package](#windows-scoop-package) is recommended.  
Alternatively you may install it manually using the
[prebuilt binaries](#windows-prebuilt-binaries).

If you're using the [Windows Subsystem for Linux][wsl], it's highly recommended
to install the [prebuilt Linux binary](#prebuilt-binaries-for-linux) instead.

Only 64-bit (`x86_64`) binaries are provided.
For other architectures and configurations you may [compile from source](#build).

A `chocolatey` package along with an `.msi` installer will be coming soon.

#### Windows: scoop package
Make sure you've [`scoop`][scoop-install] installed, and run:

```bash
scoop install ffsend
ffsend --help
```

#### Windows: Prebuilt binaries
Check out the [latest release][github-latest-release] assets for Windows binaries.
Use the `ffsend-v*-windows-x64-static` binary, to minimize the chance for issues.
If it isn't available yet, you may use an artifact from a
[previous version][github-releases] instead, until it is available.

You can use `ffsend` from the command line in the same directory:
```cmd
.\ffsend.exe --help
```

To make it globally invocable as `ffsend`, you must make the binary available in
your systems `PATH`. The easiest solution is to move it into `System32`:
```cmd
move .\ffsend.exe C:\Windows\System32\ffsend.exe
```

### FreeBSD

[» `ffsend`][freshports-ffsend]

_Note: The FreeBSD package is currently maintained by FreeBSD contributors,
and might be slightly outdated._

```sh
# Precompiled binary.
pkg install ffsend

# Compiles and installs from source.
cd /usr/ports/www/ffsend && make install
```

### Android
`ffsend` can be used on Android through Termux, install it first:
[» Termux][termux]

_Note: The Android package is currently maintained by Termux contributors,
and might be slightly outdated._

```sh
# Install package.
pkg install ffsend

ffsend help
```

### Other OS or architecture
If your system runs Docker, you can use the [docker image](#docker-image).
There are currently no other binaries or packages available.

You can [build the project from source](#build) instead.

#### Docker image
A Docker image is available for using `ffsend` running in a container.
Mount a directory to `/data`, so it's accessible for `ffsend` in the container,
and use the command as you normally would.

[» `timvisee/ffsend`][docker-hub-ffsend]

```bash
# Invoke without arguments
docker run --rm -it -v $(pwd):/data timvisee/ffsend

# Upload my-file.txt
docker run --rm -it -v $(pwd):/data timvisee/ffsend upload my-file.txt

# Download from specified link
docker run --rm -it -v $(pwd):/data timvisee/ffsend download https://send.vis.ee/#sample-share-url

# Show help
docker run --rm -it -v $(pwd):/data timvisee/ffsend help

# To update the used image
docker pull timvisee/ffsend
```

On Linux or macOS you might define a alias in your shell configuration, to make
it invocable as `ffsend`:

```bash
alias ffsend='docker run --rm -it -v "$(pwd):/data" timvisee/ffsend'
```

_Note: This implementation is limited to accessing the paths you make available
through the specified mount._

## Build
To build and install `ffsend` yourself, you meet the following requirements
before proceeding:

### Build requirements
- Runtime [requirements](#requirements)
- [`git`][git]
- [`rust`][rust] `v1.63` (MSRV) or higher (install using [`rustup`][rustup])
- [OpenSSL][openssl] or [LibreSSL][libressl] libraries/headers:
  - Linux:
    - Ubuntu, Debian and derivatives: `apt install build-essential cmake pkg-config libssl-dev`
    - CentOS/Red Hat/openSUSE: `yum install gcc gcc-c++ make cmake openssl-devel`
    - Arch: `pacman -S openssl base-devel`
    - Gentoo: `emerge -a dev-util/pkgconfig dev-util/cmake dev-libs/openssl`
    - Fedora: `dnf install gcc gcc-c++ make cmake openssl-devel`
    - Or see instructions [here](https://github.com/sfackler/rust-openssl#linux)
  - Windows:
    - Optional with `crypto-openssl` feature: See instructions here [here](https://github.com/sfackler/rust-openssl#windows-msvc)
  - macOS:
    - Optional with `crypto-openssl` feature: `brew install cmake pkg-config openssl` or see instructions [here](https://github.com/sfackler/rust-openssl#osx)
  - FreeBSD:
    - `pkg install rust gmake pkgconf python36 libxcb xclip ca_root_nss xsel-conrad`
    - It is a better idea to use & modify the existing `ffsend` port, which manages dependencies for you.

### Compile and install
Then, walk through one of the following steps to compile and install `ffsend`:

- Compile and install it directly from cargo:

  ```bash
  # Compile and install from cargo
  cargo install ffsend -f

  # Start using ffsend
  ffsend --help
  ```

- Or clone the repository and install it with `cargo`:

  ```bash
  # Clone the project
  git clone https://github.com/timvisee/ffsend.git
  cd ffsend

  # Compile and install
  cargo install --path . -f

  # Start using ffsend
  ffsend --help

  # or run it directly from cargo
  cargo run --release -- --help
  ```

- Or clone the repository and invoke the binary directly (Linux/macOS):

  ```bash
  # Clone the project
  git clone https://github.com/timvisee/ffsend.git
  cd ffsend

  # Build the project (release version)
  cargo build --release

  # Start using ffsend
  ./target/release/ffsend --help
  ```

### Compile features / use flags
Different use flags are available for `ffsend` to toggle whether to include
various features.
The following features are available, some of which are enabled by default:

| Feature         | Enabled | Description                                                |
| :-------------: | :-----: | :--------------------------------------------------------- |
| `send2`         | Default | Support for Send v2 servers                                |
| `send3`         | Default | Support for Send v3 servers                                |
| `crypto-ring`   | Default | Use ring as cryptography backend                           |
| `crypto-openssl`|         | Use OpenSSL as cryptography backend                        |
| `clipboard`     | Default | Support for copying links to the clipboard                 |
| `history`       | Default | Support for tracking files in history                      |
| `archive`       | Default | Support for archiving and extracting uploads and downloads |
| `qrcode`        | Default | Support for rendering a QR code for a share URL            |
| `urlshorten`    | Default | Support for shortening share URLs                          |
| `infer-command` | Default | Support for inferring subcommand based on binary name      |
| `no-color`      |         | Compile without color support in error and help messages   |

To enable features during building or installation, specify them with
`--features <features...>` when using `cargo`.
You may want to disable default features first using
`--no-default-features`.
Here are some examples:

```bash
# Defaults set of features with no-color, one of
cargo install --features no-color
cargo build --release --features no-color

# No default features, except required
cargo install --no-default-features --features send3,crypto-ring

# With history and clipboard support
cargo install --no-default--features --features send3,crypto-ring,history,clipboard
```

For Windows systems it is recommended to provide the `no-color` flag, as color
support in Windows terminals is flaky.

## Configuration and environment
The following environment variables may be used to configure the following
defaults. The CLI flag is shown along with it, to better describe the relation
to command line arguments:

| Variable                  | CLI flag                       | Description                                   |
| :------------------------ | :----------------------------: | :-------------------------------------------- |
| `FFSEND_HISTORY`          | `--history <FILE>`             | History file path                             |
| `FFSEND_HOST`             | `--host <URL>`                 | Upload host                                   |
| `FFSEND_TIMEOUT`          | `--timeout <SECONDS>`          | Request timeout (0 to disable)                |
| `FFSEND_TRANSFER_TIMEOUT` | `--transfer-timeout <SECONDS>` | Transfer timeout (0 to disable)               |
| `FFSEND_EXPIRY_TIME`      | `--expiry-time <SECONDS>`      | Default upload expiry time                    |
| `FFSEND_DOWNLOAD_LIMIT`   | `--download-limit <DOWNLOADS>` | Default download limit                        |
| `FFSEND_API`              | `--api <VERSION>`              | Server API version, `-` to lookup             |
| `FFSEND_BASIC_AUTH`       | `--basic-auth <USER:PASSWORD>` | Basic HTTP authentication credentials to use. |

These environment variables may be used to toggle a flag, simply by making them
available. The actual value of these variables is ignored, and variables may be
empty.

| Variable             | CLI flag        | Description                        |
| :------------------- | :-------------: | :--------------------------------- |
| `FFSEND_FORCE`       | `--force`       | Force operations                   |
| `FFSEND_NO_INTERACT` | `--no-interact` | No interaction for prompts         |
| `FFSEND_YES`         | `--yes`         | Assume yes for prompts             |
| `FFSEND_INCOGNITO`   | `--incognito`   | Incognito mode, don't use history  |
| `FFSEND_OPEN`        | `--open`        | Open share link of uploaded file   |
| `FFSEND_ARCHIVE`     | `--archive`     | Archive files uploaded             |
| `FFSEND_EXTRACT`     | `--extract`     | Extract files downloaded           |
| `FFSEND_COPY`        | `--copy`        | Copy share link to clipboard       |
| `FFSEND_COPY_CMD`    | `--copy-cmd`    | Copy download command to clipboard |
| `FFSEND_QUIET`       | `--quiet`       | Log quiet information              |
| `FFSEND_VERBOSE`     | `--verbose`     | Log verbose information            |

Some environment variables may be set at compile time to tweak some defaults.

| Variable     | Description                                                                |
| :----------- | :------------------------------------------------------------------------- |
| `XCLIP_PATH` | Set fixed `xclip` binary path when using `clipboard-bin` (Linux, &ast;BSD) |
| `XSEL_PATH`  | Set fixed `xsel` binary path when using `clipboard-bin` (Linux, &ast;BSD)  |

At this time, no configuration or _dotfile_ file support is available.
This will be something added in a later release.

### Binary for each subcommand: `ffput`, `ffget`
`ffsend` supports having a separate binaries for single subcommands, such as
having `ffput` and `ffget` just for to upload and download using `ffsend`.
This allows simple and direct commands like:
```bash
ffput my-file.txt
ffget https://send.vis.ee/#sample-share-url
```

This works for a predefined list of binary names:

- `ffput` → `ffsend upload ...`
- `ffget` → `ffsend download ...`
- `ffdel` → `ffsend delete ...`
- _This list is defined in [`src/config.rs`](./src/config.rs) as `INFER_COMMANDS`_

You can use the following methods to set up these single-command binaries:

- Create a properly named symbolic link (recommended)
- Create a properly named hard link
- Clone the `ffsend` binary, and rename it

On Linux and macOS you can use the following command to set up symbolic links in
the current directory:

```bash
ln -s $(which ffsend) ./ffput
ln -s $(which ffsend) ./ffget
```

Support for this feature is only available when `ffsend` is compiled with the
[`infer-command`](#compile-features--use-flags) feature flag.
This is usually enabled by default.
To verify support is available with an existing installation, make sure the
feature is listed when invoking `ffsend debug`.

Note that the `snap` package does currently not support this due to how this
package format works.

### Scriptability
`ffsend` is optimized for use in automated scripts. It provides some specialized
arguments to control `ffsend` without user interaction.

- `--no-interact` (`-I`): do not allow user interaction. For prompts not having
    a default value, the application will quit with an error, unless `--yes`
    or `--force` is provided.
    This should **always** be given when using automated scripting.  
    Example: when uploading a directory, providing this flag will stop the
    archive question prompt form popping up, and will archive the directory as
    default option.
- `--yes` (`-y`): assume the yes option for yes/no prompt by default.  
    Example: when downloading a file that already exists, providing this flag
    will assume yes when asking to overwrite a file.
- `--force` (`-f`): force to continue with the action, skips any warnings that
    would otherwise quit the application.  
    Example: when uploading a file that is too big, providing this flag will
    ignore the file size warning and forcefully continues.
- `--quiet` (`-q`): be quiet, print as little information as possible.  
    Example: when uploading a file, providing this flag will only output the
    final share URL.

Generally speaking, use the following rules when automating:
- Always provide `--no-interact` (`-I`).
- Provide any combination of `--yes` (`-y`) and `--force` (`-f`) for actions you
  want to complete no matter what.
- When passing share URLs along, provide the `--quiet` (`-q`) flag, when
  uploading for example.

These flags can also automatically be set by defining environment variables as
specified here:  
[» Configuration and environment](#configuration-and-environment)

Here are some examples commands in `bash`:

```bash
# Stop on error
set -e

# Upload a file
# -I: no interaction
# -y: assume yes
# -q: quiet output, just return the share link
URL=$(ffsend -Iy upload -q my-file.txt)

# Render file information
# -I: no interaction
# -f: force, just show the info
ffsend -If info $URL

# Set a password for the uploaded file
ffsend -I password $URL --password="secret"

# Use the following flags automatically from now on
# -I: no interaction
# -f: force
# -y: yes
export FFSEND_NO_INTERACT=1 FFSEND_FORCE=1 FFSEND_YES=1

# Download the uploaded file, overwriting the local variant due to variables
ffsend download $URL --password="secret"
```

For more information on these arguments, invoke `ffsend help` and check out:
[» Configuration and environment](#configuration-and-environment)

For other questions regarding automation or feature requests, be sure to
[open](https://github.com/timvisee/ffsend/issues/) an issue.

## Security
In short; the `ffsend` tool and the [Send][send] service can be considered
secure, and may be used to share sensitive files. Note though that the
created share link for an upload will allow anyone to download the file.
Make sure you don't share this link with unauthorized people.

For more detailed information on encryption, please read the rest of the
paragraphs in this security section.

_Note: even though the encryption method is considered secure, this `ffsend`
tool does not provide any warranty in any way, shape or form for files that
somehow got decrypted without proper authorization._

#### Client side encryption
`ffsend` uses client side encryption, to ensure your files are securely
encrypted before they are uploaded to the remote host. This makes it impossible
for third parties to decrypt your file without having the secret (encryption
key). The file and its metadata are encrypted using `128-bit AES-GCM`, and a
`HMAC SHA-256` signing key is used for request authentication.
This is consistent with the encryption documentation provided by the
[Send][send] service, `ffsend` is a tool for.

A detailed list on the encryption/decryption steps, and on what encryption is
exactly used can be found [here][send-encryption] in the official service
documentation.

#### Note on share link security
The encryption secret, that is used to decrypt the file when downloading,
is included in the share URL behind the `#` (hash). This secret is never sent
the remote server directly when using the share link in your browser.
It would be possible however for a webpage to load some malicious JavaScript
snippet that eventually steals the secret from the link once the page is loaded.
Although this scenario is extremely unlikely, there are some options to prevent
this from happening:

- Only use this `ffsend` tool, do not use the share link in your browser.
- Add additional protection by specifying a password using `--password` while
  uploading, or using the `password` subcommand afterwards.
- Host a secure [Send][send] service instance yourself.

A complete overview on encryption can be found in the official service
documentation [here][send-encryption].

## Help
```
$ ffsend help

ffsend 0.2.72
Tim Visee <3a4fb3964f@sinenomine.email>
Easily and securely share files from the command line.
A fully featured Send client.

The default public Send host is provided by Tim Visee, @timvisee.
Please consider to donate and help keep it running: https://vis.ee/donate

USAGE:
    ffsend [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -f, --force          Force the action, ignore warnings
    -h, --help           Prints help information
    -i, --incognito      Don't update local history for actions
    -I, --no-interact    Not interactive, do not prompt
    -q, --quiet          Produce output suitable for logging and automation
    -V, --version        Prints version information
    -v, --verbose        Enable verbose information and logging
    -y, --yes            Assume yes for prompts

OPTIONS:
    -A, --api <VERSION>                 Server API version to use, '-' to lookup [env: FFSEND_API]
        --basic-auth <USER:PASSWORD>    Protected proxy HTTP basic authentication credentials (not FxA) [env: FFSEND_BASIC_AUTH]
    -H, --history <FILE>                Use the specified history file [env: FFSEND_HISTORY]
    -t, --timeout <SECONDS>             Request timeout (0 to disable) [env: FFSEND_TIMEOUT]
    -T, --transfer-timeout <SECONDS>    Transfer timeout (0 to disable) [env: FFSEND_TRANSFER_TIMEOUT]

SUBCOMMANDS:
    upload        Upload files [aliases: u, up]
    download      Download files [aliases: d, down]
    debug         View debug information [aliases: dbg]
    delete        Delete a shared file [aliases: del, rm]
    exists        Check whether a remote file exists [aliases: e]
    generate      Generate assets [aliases: gen]
    help          Prints this message or the help of the given subcommand(s)
    history       View file history [aliases: h]
    info          Fetch info about a shared file [aliases: i]
    parameters    Change parameters of a shared file [aliases: params]
    password      Change the password of a shared file [aliases: pass, p]
    version       Determine the Send server version [aliases: v]

This application is not affiliated with Firefox or Mozilla.
```

## Special thanks
- to all `ffsend` source/package contributors
- to [Mozilla][mozilla] for building the amazing [Firefox Send][mozilla-send] service ([fork][timvisee-send])
- to everyone involved with [asciinema][asciinema] and [svg-term][svg-term] for
  providing tools to make great visual demos
- to everyone involved in all crate dependencies used

## License
This project is released under the GNU GPL-3.0 license.
Check out the [LICENSE](LICENSE) file for more information.

[usage-demo-asciinema]: https://asciinema.org/a/182225
[usage-demo-svg]: https://cdn.rawgit.com/timvisee/ffsend/6e8ef55b/res/demo.svg
[firefox]: https://firefox.com/
[git]: https://git-scm.com/
[libressl]: https://libressl.org/
[mozilla]: https://mozilla.org/
[openssl]: https://www.openssl.org/
[openssl-windows-installer]: https://u.visee.me/dl/openssl/Win64OpenSSL_Light-1_1_0j.exe
[termux]: https://termux.com/
[rust]: https://rust-lang.org/
[rustup]: https://rustup.rs/
[send]: https://github.com/timvisee/send
[mozilla-send]: https://github.com/mozilla/send
[timvisee-send]: https://github.com/timvisee/send
[send-encryption]: https://github.com/timvisee/send/blob/master/docs/encryption.md
[asciinema]: https://asciinema.org/
[svg-term]: https://github.com/marionebl/svg-term-cli
[github-releases]: https://github.com/timvisee/ffsend/releases
[github-latest-release]: https://github.com/timvisee/ffsend/releases/latest
[nix-ffsend]: https://nixos.org/nixos/packages.html?attr=ffsend&channel=nixos-unstable&query=ffsend
[fedora-ffsend]: https://src.fedoraproject.org/rpms/rust-ffsend
[aur-ffsend]: https://aur.archlinux.org/packages/ffsend/
[aur-ffsend-bin]: https://aur.archlinux.org/packages/ffsend-bin/
[aur-ffsend-git]: https://aur.archlinux.org/packages/ffsend-git/
[alpine-ffsend]: https://pkgs.alpinelinux.org/packages?name=ffsend&branch=edge
[snapcraft-ffsend]: https://snapcraft.io/ffsend
[homebrew]: https://brew.sh/
[wsl]: https://docs.microsoft.com/en-us/windows/wsl/install-win10
[docker-hub-ffsend]: https://hub.docker.com/r/timvisee/ffsend
[scoop-install]: https://scoop.sh/#installs-in-seconds
[freshports-ffsend]: https://www.freshports.org/www/ffsend
[timvisee]: https://timvisee.com/
[donate]: https://timvisee.com/donate
