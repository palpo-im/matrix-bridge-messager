# CHANGELOG.md

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure and infrastructure
- Configuration system with YAML support
- Database layer with multi-backend support (PostgreSQL, SQLite, MySQL)
- Matrix AppService integration
- SMS gateway abstraction with Mock and Twilio support
- Bridge core logic for message routing
- HTTP server with health check endpoints
- Command-line interface for administration
- Docker and docker-compose support
- Basic test framework

### Changed
- None

### Deprecated
- None

### Removed
- None

### Fixed
- None

### Security
- None

## [0.1.0] - 2026-03-02

### Added
- Initial release
- Core bridge functionality (framework)
- Matrix <-> SMS message routing (stub implementation)
- Database models and stores (stub implementation)
- Configuration management
- Logging and error handling
- Health check endpoints

[Unreleased]: https://github.com/palpo-im/matrix-bridge-message/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/palpo-im/matrix-bridge-message/releases/tag/v0.1.0
