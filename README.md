# ffsend
> Securely and easily share files from the command line.
> A fully featured [Firefox Send][send] client.

Securely and easily share files from the command line through a safe, private
and encrypted link using a single simple command.
Files are shared with [Mozillas][mozilla] [Send][send] service and may be up
to 2GB. Others are able to download these files with this tool, or through
their webbrowser.

All files are always encrypted on the client, and secrets are never shared with
the remote host. An optional password may be specified, and a default file
lifetime of 1 (up to 20) download or 24 hours is enforced to ensure your stuff
does not remain online forever.
This provides a secure platform to share your files.
Find out more about security [here](#security).

- [Features](#features)
- [Requirements](#requirements)
- [Usage](#usage)
- [Install](#install)
- [Build](#build)
- [Help](#help)
- [License](#license)

The public [Send][send] service that is used as default host is provided by
[Mozilla][mozilla].  
This application is not affiliated with [Mozilla][mozilla], [Firefox][firefox] or [Firefox Send][send].

_Note: this tool is currently in the alpha phase_

## Features
- Fully featured and friendly command line tool
- Upload and download files securely
- Always encrypted on the client
- Additional password protection and configurable download limits
- Built-in file and directory archiving and extraction
- History tracking your files for easy management
- Ability to use your own Send host
- Inspect or delete shared files
- Accurate error reporting
- Intended to be used in scripts without interaction

## Requirements
- Linux, macOS or Windows
- Internet connection when uploading or downloading

## Usage
Easily upload and download:

```bash
# Simple upload
ffsend upload my-file.txt

# Advanced upload
# - Specify a download limit of 20
# - Enter a password to encrypt the file
# - Archive the file before uploading
# - Copy the shareable URL to your clipboard
# - Open the shareable URL in your browser
ffsend upload --downloads 20 --password --archive --copy --open my-file.txt

# Upload to your own host
ffsend u -h https://example.com/ my-file.txt

# Simple download
ffsend download https://send.firefox.com/#sample-share-url
```

Inspect remote files:

```bash
# Check if a file exists
ffsend exists https://send.firefox.com/#sample-share-url

# View all file info
ffsend info https://send.firefox.com/#sample-share-url
```

Other commands include:
```bash
# View your file history
ffsend history

# Change the password after uploading
ffsend password https://send.firefox.com/#sample-share-url

# Delete a file
ffsend delete https://send.firefox.com/#sample-share-url
```

Use the `--help` flag, or see the [help][#help] section for all available subcommands.

## Install
Because `ffsend` is still in alpha, no prebuilt binaries or repositories are
available at this time.  
Build and install `ffsend` yourself using these fairly easy steps [here](#build).

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
	- macOS:
		- Using `brew`: `brew install openssl`
		- Or see instructions [here](https://github.com/sfackler/rust-openssl#osx)
	- Windows:
		- See instructions here [here](https://github.com/sfackler/rust-openssl#windows-msvc)

### Compile and install
Then, walk through one of the following steps to compile and install `ffsend`:

<!--- Compile and install it directly from cargo: -->
<!-- -->
<!--	```bash -->
<!--	# Compile and install from cargo -->
<!--	cargo install ffsend -f -->
<!-- -->
<!--	# Start using ffsend -->
<!--	ffsend --help -- -->
<!--	``` -->

- Clone the repository and install it with `cargo`:

	```bash
	# Clone the project
	git clone https://github.com/timvisee/ffsend.git
	cd ffsend/cli

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

## Help
```
$ ffsend help

ffsend 0.0.1
Tim Visee <https://timvisee.com/>
Securely and easily share files from the command line.
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

## License
This tool is released under the GNU GPL-3.0 license. Check out the
[LICENSE](LICENSE) file for more information. 

The included API library located [here](api) is intended for use in other
projects and is is released under the MIT license.
Check out the [LICENSE](api/LICENSE) file for more information.

[firefox]: https://firefox.com/
[git]: https://git-scm.com/
[libressl]: https://libressl.org/
[mozilla]: https://mozzilla.org/
[openssl]: https://www.openssl.org/
[rust]: https://rust-lang.org/
[rustup]: https://rustup.rs/
[send]: https://send.firefox.com/
