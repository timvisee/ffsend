# Release 0.1
- Set MIME type of file when downloaded 
- Remember all uploaded files, make files listable
- Incognito mode, to not remember files `--incognito`
- Automatically get owner token, from file history when setting password
- Allow file/directory archiving on upload
- Allow unarchiving on download 
- Show a simplified command list when calling `ffsend` without arguments
- Use clipboard through `xclip` on Linux if available for persistence
- Automated releases through CI
- Release binaries on GitHub
- Ubuntu PPA package
- Implement error handling everywhere properly
- Embed request errors
- Check all TODOs, solve them when possible

# Future releases
- Implement verbose logging with `-v`
- Box errors
- A status command, to check the server status using `/__version__` and
  heartbeat endpoints
- Allow piping input/output files
- Allow hiding the progress bar, and/or showing simple progress (with `-q`)
- Implement a quiet `-q` mode
- Host configuration file for host tags, to easily upload to other hosts

# Other ideas
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
