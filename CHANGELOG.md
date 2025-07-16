# Changelog

All notable changes to EDJC (Elite Dangerous Jump Calculator) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial implementation of HexChat plugin for Elite Dangerous jump calculations
- RATSIGNAL message detection and parsing
- Integration with Inara API for CMDR and ship data
- Jump calculation engine with support for neutron stars and white dwarfs
- Caching system for API responses
- Configurable output formats and thresholds
- Cross-platform build support (Windows, Linux, macOS)

### Features
- **Automatic RATSIGNAL Detection**: Monitors chat for MechaSqueak[BOT] messages
- **Real-time Data Integration**: Fetches current CMDR location and ship data from Inara
- **Smart Route Calculation**: 
  - Considers ship minimum jump range
  - Neutron star boost calculation (4x multiplier)
  - White dwarf boost calculation (1.5x multiplier)
  - Optimal route selection
- **Performance Optimizations**:
  - API response caching (5-minute default TTL)
  - Efficient regex-based message parsing
  - Background API requests
- **Configuration Management**:
  - TOML configuration file support
  - Customizable result formatting
  - Adjustable distance thresholds
  - Debug logging options

### Technical Details
- **Language**: Rust 2021 edition
- **Architecture**: HexChat C-compatible plugin (cdylib)
- **Dependencies**: 
  - reqwest for HTTP API calls
  - serde for JSON serialization
  - moka for response caching
  - regex for message parsing
  - anyhow for error handling
- **Platform Support**: Windows, Linux, macOS

## [0.1.0] - 2025-01-17

### Added
- Initial project setup and structure
- Core plugin architecture
- Inara API client implementation
- Jump calculation algorithms
- HexChat plugin interface
- Configuration system
- Build system and CI/CD setup
- Documentation and examples

### Project Structure
```
src/
├── lib.rs              # Main plugin entry point
├── hexchat.rs          # HexChat FFI bindings
├── inara.rs            # Inara API client
├── jump_calculator.rs  # Jump calculation logic
├── config.rs           # Configuration management
└── types.rs            # Shared data structures
```

### Dependencies
- reqwest 0.11 (HTTP client)
- serde 1.0 (Serialization)
- regex 1.0 (Pattern matching)
- anyhow 1.0 (Error handling)
- log 0.4 (Logging)
- moka 0.12 (Caching)
- chrono 0.4 (Date/time)

### Build System
- Cargo.toml configuration
- Cross-platform build script
- VS Code tasks integration
- GitHub Actions CI/CD (planned)

### Documentation
- Comprehensive README.md
- API documentation
- Configuration examples
- Contributing guidelines
- MIT License

---

## Release Notes Format

Each release includes:
- **Added**: New features and capabilities
- **Changed**: Modifications to existing functionality
- **Deprecated**: Features marked for removal
- **Removed**: Features that have been removed
- **Fixed**: Bug fixes and corrections
- **Security**: Security-related improvements

## Versioning Strategy

- **Major** (x.0.0): Breaking changes, major new features
- **Minor** (0.x.0): New features, backwards compatible
- **Patch** (0.0.x): Bug fixes, small improvements

## Development Milestones

### Phase 1: Core Functionality (v0.1.x)
- [x] Basic plugin structure
- [x] RATSIGNAL message parsing
- [x] Inara API integration
- [x] Jump calculation engine
- [x] Configuration system

### Phase 2: Enhanced Features (v0.2.x)
- [ ] Advanced routing algorithms
- [ ] Multiple route suggestions
- [ ] Fuel calculation estimates
- [ ] Time estimation features
- [ ] Enhanced error handling

### Phase 3: User Experience (v0.3.x)
- [ ] Interactive configuration
- [ ] Plugin management commands
- [ ] Status indicators
- [ ] Performance monitoring
- [ ] User preferences

### Phase 4: Community Features (v1.0.x)
- [ ] Plugin marketplace compatibility
- [ ] Community route sharing
- [ ] Statistics and analytics
- [ ] Advanced customization
- [ ] Stable API for extensions

## Known Issues

### Current Limitations
- Requires manual Inara API key configuration
- Basic route calculation (no pathfinding through actual stellar data)
- Limited to HexChat IRC client
- Manual plugin installation required

### Planned Improvements
- GUI configuration tool
- Automatic stellar database updates
- Support for other IRC clients
- Installer packages for easy setup

## API Compatibility

### Inara API
- **Version**: v1
- **Rate Limit**: 1 request per second
- **Caching**: 5-minute default TTL
- **Required Data**: CMDR location, ship information, system coordinates

### HexChat Plugin API
- **Version**: 2.x compatible
- **Interface**: C FFI
- **Memory Management**: Manual (Rust handles internally)
- **Event Handling**: Message hooks

## Performance Metrics

### Target Performance
- **Startup Time**: < 1 second
- **Response Time**: < 2 seconds for jump calculations
- **Memory Usage**: < 50MB during normal operation
- **Cache Hit Rate**: > 80% for system lookups

### Monitoring
- Log-based performance tracking
- Cache hit/miss statistics
- API response time monitoring
- Error rate tracking
