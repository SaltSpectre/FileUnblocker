# SaltSpectre's File Unblocker

A secure, high-performance Windows utility that removes Window's "[Mark of the Web](https://en.wikipedia.org/wiki/Mark_of_the_Web)" Zone.Identifier alternate data stream to unblock files downloaded from the internet. 

## Background 

This utility does the equivalent to using the `Unblock-File` [PowerShell cmdlet](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.utility/unblock-file) or opening the properties for a file, ticking the "Unblock" checkbox, and applying the change. I wanted a tool that I could use in the File Explorer context menu or to easily unblock all the files in a directory, and neither of the aformentioned options directly supported that. Even manually adding a context menu entry that performed `Unblock-File` turned out not to be an option due to Windows Defender flagging the activity as potentially malicious. Enter SaltSpectre's File Unblocker utility:

## Features

### Core Functionality
- **Fast file unblocking**: Remove Zone.Identifier Alternate Data Stream (ADS) from individual files or entire directories
- **Recursive processing**: Automatically traverse subdirectories with detailed progress tracking  
- **Smart elevation**: Automatic UAC elevation only when needed, with secure argument handling
- **Comprehensive logging**: When used with the --log parameter. See Usage section below.

### Safety Guardrails
- **Path validation**: Prevents directory traversal attacks and validates all file paths
- **Safe system paths**: Automatically skips dangerous system directories
- **Argument injection protection**: Secure command-line argument escaping
- **Memory safety**: Written in Rust!

### User Experience  
- **CLI interface**: Modern command-line parsing with help and version info
- **Context menu integration**: Right-click support in Windows Explorer when installed via the MSI
- **Detailed statistics**: Reports on files processed, unblocked, failed, etc.

## Installation

### MSI Installer (Recommended)
1. Download the latest MSI installer from the [Releases](https://github.com/SaltSpectre/cs-SaltSpectre-Unblocker/releases) page.
2. Run the MSI installer
3. Choose per-user or system-wide installation when prompted. The *per-user* installation does not require administrative privileges to install.
4. Context menu integration is automatically configured

### Portable Executable
1. Download `portable-unblocker-x64.exe` or `portable-unblocker-arm64.exe` from [Releases](https://github.com/SaltSpectre/cs-SaltSpectre-Unblocker/releases)
2. Place executable in desired location
3. No context menu integration (manual command-line use only)

## Usage

```cmd
unblocker.exe "C:\path\to\file.exe"
unblocker.exe --verbose "C:\path\to\directory"
unblocker.exe --log "log.txt" "C:\Downloads"
```

### Context Menu (MSI installer only)
Right-click files/directories → "Unblock file with File Unblocker" / "Unblock directory with File Unblocker"

---

## Building

### Build Script method
```powershell
cd rust
.\build-windows.ps1 -Clean
```

### Manual method

**For x86_64/amd64:**
```cmd
cargo build --release --target x86_64-pc-windows-msvc
```

**For arm64:**
```cmd
cargo build --release --target aarch64-pc-windows-msvc
```

---

## Contributing

This project is proudly open source under the MIT License. Feel free to open a pull request with your code changes. Please ensure that you are following the arcitectual model of this project.