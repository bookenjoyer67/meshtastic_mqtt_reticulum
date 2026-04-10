# Changelog

All notable changes to the Meshtastic MQTT Reticulum Bridge project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure
- MQTT client implementation for Meshtastic
- Reticulum interface support (TCP, UDP, Serial, KISS, MQTT, Kaonic)
- Message conversion between Meshtastic protobuf and Reticulum packets
- Configuration system with JSON support
- Logging system with multiple output formats
- Basic error handling and reconnection logic

### Changed
- N/A (initial release)

### Deprecated
- N/A (initial release)

### Removed
- N/A (initial release)

### Fixed
- N/A (initial release)

### Security
- Initial security implementation with Reticulum's end-to-end encryption
- MQTT authentication support
- Basic input validation and sanitization

## [0.1.0] - 2026-03-31

### Added
- Initial release of Meshtastic MQTT Reticulum Bridge
- Support for connecting to Meshtastic MQTT brokers
- Support for multiple Reticulum interface types
- Basic message routing between networks
- Configuration file support
- Logging to file and console

### Features
- **MQTT Integration**: Connect to Meshtastic MQTT brokers
- **Reticulum Interfaces**: TCP, UDP, Serial, KISS, MQTT, Kaonic
- **Message Conversion**: Convert between Meshtastic and Reticulum formats
- **Configuration**: JSON-based configuration system
- **Logging**: Structured logging with multiple levels
- **Error Handling**: Automatic reconnection and error recovery

### Known Issues
- MQTT interface for Reticulum is experimental
- Some edge cases in message conversion may not be handled
- Performance optimizations needed for high-volume traffic
- Limited testing on all platform combinations

### Security Notes
- Uses Reticulum's built-in encryption for all Reticulum traffic
- MQTT connections support TLS (when configured)
- Basic authentication for MQTT connections
- No known security vulnerabilities at this time

## Future Releases

### Planned for 0.2.0
- Web interface for administration
- Advanced message routing rules
- Plugin system for extensibility
- Performance optimizations
- Additional message type support
- Improved error handling and recovery

### Planned for 0.3.0
- Docker container support
- Kubernetes deployment manifests
- Prometheus metrics exporter
- Grafana dashboards
- Advanced monitoring and alerting
- API for external integration

### Long-term Goals
- Support for additional mesh networking protocols
- Advanced routing algorithms
- Geographic message routing
- Quality of Service (QoS) support
- Mobile application
- Cloud deployment options

## Versioning Policy

This project uses [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for new functionality in a backward-compatible manner
- **PATCH** version for backward-compatible bug fixes

## Upgrade Instructions

### From Unreleased to 0.1.0
This is the initial release. No upgrade instructions are needed.

### General Upgrade Guidelines
1. Backup your configuration file
2. Check the changelog for breaking changes
3. Update the binary/package
4. Review and update configuration if needed
5. Test the new version in a non-production environment

## Deprecation Policy

Features marked as deprecated will:
1. Continue to work for at least one minor version
2. Show warnings when used
3. Be removed in a future major version
4. Have migration paths documented

## Contributing to the Changelog

When adding entries to the changelog, please follow these guidelines:

1. Add entries under the appropriate version section
2. Use the categories: Added, Changed, Deprecated, Removed, Fixed, Security
3. Be concise but descriptive
4. Reference issue numbers when applicable
5. Include relevant details for security fixes

## Acknowledgments

- The Meshtastic team for their amazing open-source mesh networking platform
- The Reticulum developers for their secure networking stack
- All contributors and testers who have helped with the project