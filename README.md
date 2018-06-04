[![Build status on Travis CI][travis-master-badge]][travis-link]
[![Newest release on crates.io][crate-version-badge]][crate-link]
[![Number of downloads on crates.io][crate-download-badge]][crate-link]
[![Project license][crate-license-badge]](LICENSE)

[crate-link]: https://crates.io/crates/ffsend
[crate-download-badge]: https://img.shields.io/crates/d/ffsend.svg
[crate-version-badge]: https://img.shields.io/crates/v/ffsend.svg
[crate-license-badge]: https://img.shields.io/crates/l/ffsend.svg
[travis-master-badge]: https://travis-ci.org/timvisee/ffsend.svg?branch=master
[travis-link]: https://travis-ci.org/timvisee/ffsend

# ffsend [WIP]
> Easily and securely share files from the command line.
> A fully featured [Firefox Send][send] client.

Easily and securely share files and directories from the command line through a
safe, private and encrypted link using a single simple command.
Files are shared using the [Send][send] service and may be up
to 2GB. Others are able to download these files with this tool, or through
their webbrowser.

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
- [Install](#install)
- [Build](#build)
- [Configuration and environment](#configuration-and-environment)
- [Security](#security)
- [Help](#help)
- [Special thanks](#special-thanks)
- [License](#license)

The public [Send][send] service that is used as default host is provided by
[Mozilla][mozilla].  
This application is not affiliated with [Mozilla][mozilla], [Firefox][firefox]
or [Firefox Send][send] in any way.

_Note: this tool is currently in the alpha phase_

## Features
- Fully featured and friendly command line tool
- Upload and download files and directories securely
- Always encrypted on the client
- Additional password protection and configurable download limits
- Built-in file and directory archiving and extraction
- History tracking your files for easy management
- Ability to use your own Send host
- Inspect or delete shared files
- Accurate error reporting
- Low memory footprint, due to encryption, download and upload streaming
- Intended to be used in scripts without interaction

For a list of upcoming features and ideas, take a look at the
[ROADMAP](ROADMAP.md) file.

## Usage
Easily upload and download:

```bash
# Simple upload
$ ffsend upload my-file.txt
Share link: https://send.firefox.com/#sample-share-url

# Advanced upload
# - Specify a download limit of 20
# - Enter a password to encrypt the file
# - Archive the file before uploading
# - Copy the shareable link to your clipboard
# - Open the shareable link in your browser
$ ffsend upload --downloads 20 --password --archive --copy --open my-file.txt
Password: ******
Share link: https://send.firefox.com/#sample-share-url

# Upload to your own host
$ ffsend u -h https://example.com/ my-file.txt
Share link: https://example.com/#sample-share-url

# Simple download
$ ffsend download https://send.firefox.com/#sample-share-url
```

Inspect remote files:

```bash
# Check if a file exists
$ ffsend exists https://send.firefox.com/#sample-share-url
Exists: yes

# Fetch remote file info
$ ffsend info https://send.firefox.com/#sample-share-url
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
#  LINK                                        EXPIRY  OWNER TOKEN
1  https://send.firefox.com/#sample-share-url  23h57m  eea9f544f6d5df8a5afd
2  https://send.firefox.com/#other-sample-url  17h38m  1e9fef63fee3baaf54ce
3  https://example.com/#sample-share-url       37m30s  8eb28bc1bc85cfdab0e4

# Change the password after uploading
$ ffsend password https://send.firefox.com/#sample-share-url
Password: ******

# Delete a file
$ ffsend delete https://send.firefox.com/#sample-share-url
```

Use the `--help` flag, `help` subcommand, or see the [help](#help) section for
all available subcommands.

## Requirements
- Linux, macOS or Windows
- A terminal :sunglasses:
- Linux specific:
  - `xclip` for clipboard support (optional)
    - Ubuntu/Debian: `apt install xclip`
    - CentOS/Red Hat/openSUSE/Fedora: `yum install xclip`
    - Arch: `pacman -S xclip`
- Internet connection for uploading and downloading

## Install
<!-- Before installing, make sure you meet all requirements listed
[here](#requirements) -->

Because `ffsend` is still in alpha, only limited installation options are
available right now.  

A set of pre-build binaries for Linux and macOS can be found as asset of the
[latest release][github-latest-release]. When downloading such release, mark
the binary as executable using `chmod a+x ffsend`, and move it into `/usr/bin/`.

A Windows binary and packaged versions for various Linux distributions is
currently being worked on.

It is recommended to build and install `ffsend` yourself using these fairly
easy steps [here](#build).

## Build
To build and install `ffsend` yourself, you meet the following requirements
before proceeding:

### Build requirements
- Regular [requirements](#requirements)
- [`git`][git]
- [`rust`][rust] `v1.26` or higher (install using [`rustup`][rustup])
- [OpenSSL][openssl] or [LibreSSL][libressl] libraries and headers must be available
	- Linux:
		- Ubuntu/Debian: `apt install pkg-config libssl-dev`
		- CentOS/Red Hat/openSUSE: `yum install openssl-devel`
		- Arch: `pacman -S openssl`
		- Fedora: `dnf install openssl-devel`
		- Or see instructions [here](https://github.com/sfackler/rust-openssl#linux)
	- macOS:
		- Using `brew`: `brew install openssl`
		- Or see instructions [here](https://github.com/sfackler/rust-openssl#osx)
	- Windows:
		- See instructions here [here](https://github.com/sfackler/rust-openssl#windows-msvc)

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
	cargo install -f

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

| Feature     | Enabled | Description                                                |
| :---------: | :-----: | :--------------------------------------------------------- |
| `clipboard` | Default | Support for copying links to the clipboard                 |
| `history`   | Default | Support for tracking files in history                      |
| `archive`   | Default | Support for archiving and extracting uploads and downloads |
| `no-color`  |         | Compile without color support in error and help messages   |

To enable features during building or installation, specify them with
`--features <features, >` when using `cargo`.
You may want to disable alisl default features first using
`--no-default-features`.
Here are some examples:

```bash
# Defaults set of features with no-color, one of
cargo install --features no-color
cargo build --release --features no-color

# None of the features
cargo install --no-default-features

# Only history and clipboard support
cargo install --no-default--features --features history,clipboard
```

## Configuration and environment
The following environment variables may be used to configure the following
defaults. The CLI flag is shown along with it, to better describe the relation
to command line arguments:

| Variable         | CLI flag           | Description       |
| :--------------- | :----------------: | :---------------- |
| `FFSEND_HISTORY` | `--history <FILE>` | History file path |
| `FFSEND_HOST`    | `--host <URL>`     | Upload host       |

These environment variables may be used to toggle a flag, simply by making them
available. The actual value of these variables is ignored, and variables may be
empty.

| Variable             | CLI flag        | Description                       |
| :------------------- | :-------------: | :-------------------------------- |
| `FFSEND_FORCE`       | `--force`       | Force operations                  |
| `FFSEND_NO_INTERACT` | `--no-interact` | No interaction for prompts        |
| `FFSEND_YES`         | `--yes`         | Assume yes for prompts            |
| `FFSEND_INCOGNITO`   | `--incognito`   | Incognito mode, don't use history |
| `FFSEND_OPEN`        | `--open`        | Open share link of uploaded file  |
| `FFSEND_ARCHIVE`     | `--archive`     | Archive files uploaded            |
| `FFSEND_EXTRACT`     | `--extract`     | Extract files downloaded          |
| `FFSEND_COPY`        | `--copy`        | Copy share link to clipboard      |
| `FFSEND_VERBOSE`     | `--verbose`     | Log verbose information           |

At this time, no configuration or _dotfile_ file support is available.
This will be something added in a later release.

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
key). The file and it's metadata are encrypted using `128-bit AES-GCM`, and a
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

ffsend 0.0.7
Tim Visee <https://timvisee.com/>
Easily and securely share files from the command line.
A fully featured Firefox Send client.

USAGE:
    ffsend [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -f, --force          Force the action, ignore warnings
    -h, --help           Prints help information
    -i, --incognito      Don't update local history for actions
    -I, --no-interact    Not interactive, do not prompt
    -V, --version        Prints version information
    -v, --verbose        Enable verbose information and logging
    -y, --yes            Assume yes for prompts

OPTIONS:
    -H, --history <FILE>    Use the specified history file [env: FFSEND_HISTORY]

SUBCOMMANDS:
    upload        Upload files [aliases: u, up]
    download      Download files [aliases: d, down]
    debug         View debug information [aliases: dbg]
    delete        Delete a shared file [aliases: del]
    exists        Check whether a remote file exists [aliases: e]
    help          Prints this message or the help of the given subcommand(s)
    history       View file history [aliases: h]
    info          Fetch info about a shared file [aliases: i]
    parameters    Change parameters of a shared file [aliases: params]
    password      Change the password of a shared file [aliases: pass, p]

The public Send service that is used as default host is provided by Mozilla.
This application is not affiliated with Mozilla, Firefox or Firefox Send.
```

## Special thanks
- to [Mozilla][mozilla] for building and hosting the amazing
  [Firefox Send][send] service
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
[mozilla]: https://mozzilla.org/
[openssl]: https://www.openssl.org/
[rust]: https://rust-lang.org/
[rustup]: https://rustup.rs/
[send]: https://send.firefox.com/
[send-encryption]: https://github.com/mozilla/send/blob/master/docs/encryption.md
[asciinema]: https://asciinema.org/
[svg-term]: https://github.com/marionebl/svg-term-cli
[github-latest-release]: https://github.com/timvisee/ffsend/releases/latest
