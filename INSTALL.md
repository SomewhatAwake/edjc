# EDJC Installation Guide

## Quick Start

### Prerequisites
- HexChat IRC client installed
- An EDSM API key ([Get one here](https://www.edsm.net/en/settings/api))

### Installation Steps

1. **Download the Plugin**
   - Download `edjc.dll` from the [latest release](https://github.com/username/edjc/releases)
   - Or build from source (see Development section below)

2. **Install the Plugin**
   ```
   Copy edjc.dll to your HexChat plugins directory:
   - Windows: %APPDATA%\HexChat\addons\
   - Linux: ~/.config/hexchat/addons/
   - macOS: ~/Library/Application Support/HexChat/addons/
   ```

3. **Configure the Plugin**
   ```
   Copy edjc.toml.example to your config directory:
   - Windows: %APPDATA%\EDJC\edjc.toml
   - Linux/macOS: ~/.config/edjc/edjc.toml
   
   Edit the config file and add:
   edsm_api_key = "your_api_key_here"
   cmdr_name = "YourCMDRName"
   ```

4. **Restart HexChat**
   - Close and reopen HexChat
   - The plugin should load automatically

### Verification

To verify the plugin is working:
1. Open HexChat
2. Check for "EDJC plugin initialized successfully" in the server window
3. Join a channel where RATSIGNAL messages appear
4. Wait for a RATSIGNAL message from MechaSqueak[BOT]

## Building from Source

### Prerequisites
- Rust toolchain (rustup recommended)
- Git
- Visual Studio Build Tools (Windows) or build-essential (Linux)

### Build Steps

1. **Clone the Repository**
   ```bash
   git clone https://github.com/username/edjc.git
   cd edjc
   ```

2. **Build the Plugin**
   ```bash
   cargo build --release
   ```

3. **Locate the Plugin**
   ```
   The plugin will be in:
   target/release/deps/edjc.dll (Windows)
   target/release/deps/libedjc.so (Linux)
   target/release/deps/libedjc.so (macOS)
   ```

4. **Copy to HexChat Directory**
   ```bash
   # Windows
   copy target\release\deps\edjc.dll "%APPDATA%\HexChat\addons\"
   
   # Linux/macOS
   cp target/release/deps/libedjc.so ~/.config/hexchat/addons/edjc.so
   ```

## Configuration Options

### Basic Configuration
```toml
# Your EDSM API key (required)
edsm_api_key = "your_api_key_here"

# Cache timeout in seconds (default: 300)
cache_timeout_seconds = 300

# Enable debug logging (default: false)
debug_mode = false
```

### Advanced Configuration
```toml
# Distance thresholds for route suggestions
neutron_highway_threshold_ly = 500.0
white_dwarf_threshold_ly = 150.0

# Customize output format
result_format = "🚀 {jumps} jumps to {system} ({distance:.1}ly) via {route}"

# Show additional estimates
show_fuel_estimates = false
show_time_estimates = false
```

## Troubleshooting

### Plugin Not Loading
1. **Check HexChat Version**: Ensure you're using HexChat 2.12 or later
2. **Check File Location**: Verify the DLL is in the correct addons directory
3. **Check Dependencies**: Ensure Visual C++ Redistributable is installed (Windows)
4. **Check Logs**: Look for error messages in HexChat's server window

### API Key Issues
1. **Verify Key**: Test your API key at [EDSM API](https://www.edsm.net/en/settings/api)
2. **Check Format**: Ensure the key is quoted in the config file
3. **Check Permissions**: Verify the config file is readable

### No Jump Calculations
1. **Check Config**: Ensure edsm_api_key is set correctly
2. **Check Network**: Verify internet connectivity
3. **Check Messages**: Ensure RATSIGNAL messages are from MechaSqueak[BOT]
4. **Check Debug Mode**: Enable debug_mode = true for detailed logging

### Common Error Messages

**"EDSM API key is required but not configured"**
- Solution: Add your API key to the config file

**"No CMDR location data received from EDSM"**
- Solution: Verify your API key and check that your CMDR profile has location data accessible

**"System not found"**
- Solution: Check system name spelling, some systems may not be in EDSM's database

## Development Setup

### IDE Setup
1. Install [VS Code](https://code.visualstudio.com/)
2. Install the [Rust Analyzer extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
3. Open the project folder in VS Code

### Testing
```bash
# Run unit tests
cargo test

# Check for issues
cargo clippy

# Format code
cargo fmt
```

### Contributing
See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

## Support

- **Issues**: [GitHub Issues](https://github.com/username/edjc/issues)
- **Documentation**: [Project Wiki](https://github.com/username/edjc/wiki)
- **Chat**: #fuel-rats on IRC (irc.fuelrats.com)

## Legal

This project is not affiliated with Frontier Developments or Elite: Dangerous. It is a community-created tool licensed under the MIT License.
