[![Build](https://github.com/SomewhatAwake/edjc/actions/workflows/ci.yml/badge.svg)](https://github.com/SomewhatAwake/edjc/actions/workflows/ci.yml) [![Docs](https://github.com/SomewhatAwake/edjc/actions/workflows/jekyll-gh-pages.yml/badge.svg)](https://github.com/SomewhatAwake/edjc/actions/workflows/jekyll-gh-pages.yml) ![Static Badge](https://img.shields.io/badge/my%20pain-immeasurable-red)


# EDJC - Elite Dangerous Jump Calculator (HexChat Plugin)

A HexChat plugin written in Rust that automatically calculates optimal jump routes in Elite: Dangerous when RATSIGNAL messages are detected in chat.

## Features

- **Automatic RATSIGNAL Detection**: Monitors chat for MechaSqueak[BOT] RATSIGNAL messages
- **Smart Route Calculation**: Considers neutron stars (4x boost) and white dwarfs (1.5x boost)
- **User-Configured Ship**: Manual ship jump range configuration for accuracy
- **Caching**: Intelligent caching of API responses for better performance
- **Configurable**: Customizable output formats and calculation thresholds

## Installation

### Prerequisites

- HexChat IRC client
### Quick Install

1. **Download the Plugin**:
   - Download `edjc.dll` from the releases page
   - Or build from source (see [INSTALL.md](INSTALL.md))

2. **Install to HexChat**:
   ```
   Copy edjc.dll to: %APPDATA%\HexChat\addons\ (Windows)
   ```

3. **Configure**:
   ```
   Copy edjc.toml.example to: %APPDATA%\EDJC\edjc.toml
   Set your CMDR name and ship's laden jump range
   ```

4. **Restart HexChat**

For detailed installation instructions, see [INSTALL.md](INSTALL.md).

## Usage

The plugin automatically triggers when it detects a RATSIGNAL message from `MechaSqueak[BOT]`. 

### Example Trigger Message
```
RATSIGNAL Case #3 PC ODY – CMDR ResponsibleFuelManagement – System: "Far Flung System" (Brown dwarf 123 LY from Fuelum) – Language: English (United States) (en-US) (ODY_SIGNAL)
```

### Example Output
```
🚀 Case #3: 12 jumps to Far Flung System (289.4ly) via neutron highway route (from Sol with 35.0ly range)
```

## Configuration Options

The `edjc.toml` configuration file supports the following options:

```toml
# Your CMDR name (for display purposes)
cmdr_name = "YOUR_CMDR_NAME"

# Ship configuration
[ship]
laden_jump_range = 35.0

# Cache timeout in seconds (default: 300)
cache_timeout_seconds = 300

# Enable debug logging (default: false)
debug_mode = false

# Distance thresholds for route suggestions
neutron_highway_threshold_ly = 500.0
white_dwarf_threshold_ly = 150.0

# Result format string
result_format = "🚀 {jumps} jumps to {system} ({distance:.1}ly) via {route}"

# Show additional estimates
show_fuel_estimates = false
show_time_estimates = false
```

### Format Placeholders

The `result_format` string supports the following placeholders:

- `{jumps}` - Number of jumps required
- `{system}` - Destination system name
- `{distance}` - Total distance in light years
- `{route}` - Route type (direct, neutron highway, white dwarf assisted)
- `{from}` - Origin system name
- `{to}` - Destination system name

## How It Works

1. **Message Detection**: The plugin monitors all chat messages for the RATSIGNAL pattern
2. **System Extraction**: Parses the system name from the RATSIGNAL message
3. **Data Retrieval**: Fetches system coordinates from EDSM API
4. **Route Calculation**: Calculates optimal route considering:
   - User-configured ship laden jump range
   - Neutron star locations (4x jump range multiplier)
   - White dwarf locations (1.5x jump range multiplier)
5. **Result Display**: Shows the calculation result as a HexChat notice

## Development

### Project Structure

```
src/
├── lib.rs              # Main plugin entry point
├── hexchat.rs          # HexChat FFI bindings
├── edsm.rs             # EDSM API client
├── inara.rs            # Legacy Inara API client (unused)
├── jump_calculator.rs  # Jump calculation logic
├── config.rs           # Configuration management
└── types.rs            # Shared data structures
```

### Building for Development

```bash
# Build with debug symbols
cargo build

# Run tests
cargo test

# Check for linting issues
cargo clippy

# Format code
cargo fmt
```

### API Integration

This plugin uses the [EDSM API](https://www.edsm.net/api-v1/) to fetch:

- System coordinates and stellar data
- Distance calculations between systems

EDSM is free to use and requires no API keys or registration. API responses are cached for 1 hour by default to reduce API calls and improve performance.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This plugin is not affiliated with or endorsed by Frontier Developments or the creators of Elite: Dangerous. It is a community-created tool for enhancing the gaming experience.

## Support

- **Issues**: [GitHub Issues](https://github.com/SomewhatAwake/edjc/issues)
- **Elite Dangerous**: [Official Website](https://www.elitedangerous.com/)

## Acknowledgments

- [Fuel Rats](https://fuelrats.com/) for their amazing rescue service
- [EDSM](https://www.edsm.net/) for providing free system data
- [HexChat](https://hexchat.github.io/) for the plugin platform
- Elite: Dangerous community for inspiration and support
