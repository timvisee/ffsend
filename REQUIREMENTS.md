# Requirements
This tool has some additional compilation requirements besides the Rust
toolchain.

- Rust 1.26 or above (verify this)

## Requirements
- OpenSSL development files
  - Ubuntu package: `libssl-dev`
- On Linux, `xorg` development packages (for `clipboard` feature)
  - Ubuntu package: `xorg-dev`

## Environment variables
- `FFSEND_HOST`: upload host (string)
- `FFSEND_FORCE`: upload host (present/boolean)
- `FFSEND_NO_INTERACT`: upload host (present/boolean)
- `FFSEND_YES`: upload host (present/boolean)
- `FFSEND_HISTORY`: history file path (string)
- `FFSEND_INCOGNITO`: incognito mode (present/boolean)
- `FFSEND_OPEN`: open an uploaded file (present/boolean)
- `FFSEND_ARCHIVE`: enable file archival (present/boolean)
- `FFSEND_COPY`: copy share URL to clipboard (present/boolean)

