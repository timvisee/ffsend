# Alpha release 0.0.1 (private feedback)
The first release used for gathering feedback on the application by selected
people.

Features:
- Write complete README
- Polish command outputs, make it consistent (format, color)
- Automated releases through CI
- Release binaries on GitHub
- Ubuntu PPA package
- Gentoo portage package
- Arch AUR package
- Windows, macOS and Redox support
- Implement verbose logging with `-v`
- Make use of stdout and stderr consistent
- Allow empty owner token for info command
- Check and validate all errors, are some too verbose?

# Beta release 0.1 (public)
The first public release.

Features:
- Do not write archives to the disk (temporarily), stream their contents
- Implement error handling everywhere properly
- Extract utility module
- Embed/wrap request errors with failure
- Box errors
- Allow piping input/output files
- Allow hiding the progress bar, and/or showing simple progress (with `-q`)
- Implement a quiet `-q` mode
- Update all dependencies
- Check all TODOs, solve them when possible
- Allow multi file uploads (select more than one file or directory)

# Future releases
- Color usage flag
- A status command, to check the server status using `/__version__` and
  heartbeat endpoints
- Host configuration file for host tags, to easily upload to other hosts

# Other ideas
- Check if extracting an archive overwrites files 
- Flag to disable logging to stderr
- Rework encrypted reader/writer
- API actions contain duplicate code, create centralized functions
- Only allow file extension renaming on upload with `-f` flag
- Quick upload/download without `upload` or `download` subcommands?
- Flag to explicitly delete file after download
- Allow file deletion by consuming all download slots
- Download to a temporary location first
- Document all code components
- Dotfile for default properties
- Generate man pages
- Rename host to server?
- Ask to add MIME extension to downloaded files without one on Windows
- Fetch max file size from `server/jsconfig.js`
- Define a redirect policy (allow setting max redirects)
- Support servers that are hosted on a sub path (URL builder resets path)
