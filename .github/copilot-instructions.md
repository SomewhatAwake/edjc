<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

# EDJC (Elite Dangerous Jump Calculator) - HexChat Plugin

## Project Overview
This is a Rust-based HexChat plugin that calculates optimal jump routes in Elite: Dangerous. The plugin integrates with the EDSM API to fetch real-time CMDR locations and system data.

## Key Features
- Automatic RATSIGNAL message detection and parsing
- Integration with EDSM API for CMDR location and system data
- Jump calculation considering neutron stars (4x boost) and white dwarfs (1.5x boost)
- Caching system for API responses
- Configurable output formats and thresholds

## Architecture
- `lib.rs` - Main plugin entry point and HexChat integration
- `hexchat.rs` - HexChat FFI bindings and utilities
- `edsm.rs` - EDSM API client with caching
- `jump_calculator.rs` - Core jump calculation logic
- `config.rs` - Configuration management
- `types.rs` - Shared data structures and types

## Development Guidelines
- Use `anyhow::Result` for error handling
- Implement comprehensive logging with the `log` crate
- Cache API responses using `moka` for performance
- Follow Rust naming conventions and documentation standards
- Write unit tests for core functionality
- Use `serde` for serialization/deserialization

## API Integration
The plugin uses the EDSM API (https://www.edsm.net/api-v1/) to fetch:
- CMDR current location and status
- System coordinates and stellar data

## HexChat Plugin Structure
This is a native HexChat plugin that:
- Exports required C-compatible functions (`hexchat_plugin_init`, `hexchat_plugin_deinit`)
- Hooks into chat message events
- Provides response messages as HexChat notices

## Configuration
- Config file location: `%APPDATA%/EDJC/edjc.toml` (Windows) or `~/.config/edjc/edjc.toml` (Unix)
- Requires EDSM API key for accessing commander location data
- Supports customizable result formats and distance thresholds
