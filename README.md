# matrix-bridge-messager

A Matrix <-> SMS/Message bridge written in Rust.

[中文文档](README_CN.md)

Maintainer: `Palpo Team`  
Contact: `chris@acroidea.com`

## Overview

- Rust-only implementation for Matrix <-> SMS/Message bridging
- Matrix appservice + SMS gateway bridge core
- HTTP endpoints for health/status/metrics and provisioning
- Database backends: PostgreSQL, SQLite, and MySQL (feature-gated)
- Dockerfile for local build and container runtime

## Status

🚧 **Under Active Development** - This project is in early development stages.

## Features (Planned)

- [x] Project structure and configuration
- [ ] Matrix AppService integration
- [ ] SMS gateway support (Twilio, AWS SNS)
- [ ] Bidirectional message bridging
- [ ] Media message support
- [ ] Read receipts and typing notifications
- [ ] Contact synchronization
- [ ] Multi-database support

## Repository Layout

- `src/`: bridge implementation
- `config/config.sample.yaml`: sample configuration
- `migrations/`: database migrations
- `Dockerfile`: multi-stage container build (planned)

## Prerequisites

- Rust toolchain (compatible with the project; Docker build uses Rust 1.93)
- A Matrix homeserver configured for appservices
- SMS gateway credentials (Twilio or AWS SNS)
- Database: PostgreSQL, SQLite, or MySQL

## Quick Start (Local)

1. Create your config file:

```bash
cp config/config.sample.yaml config.yaml
```

2. Set the required values in `config.yaml`:
   - `bridge.domain`
   - `bridge.homeserver_url`
   - `database.url`
   - `message.gateway_type` and corresponding gateway credentials
   - registration values (`as_token`, `hs_token`)

3. Run:

```bash
cargo check
cargo run
```

## Configure SMS Gateway

### Twilio

1. Sign up at https://www.twilio.com
2. Get your Account SID and Auth Token from the console
3. Purchase or use a Twilio phone number
4. Configure in `config.yaml`:

```yaml
message:
  gateway_type: "twilio"
  twilio:
    account_sid: "YOUR_ACCOUNT_SID"
    auth_token: "YOUR_AUTH_TOKEN"
    phone_number: "+1234567890"
```

### AWS SNS

1. Set up AWS credentials with SNS permissions
2. Configure in `config.yaml`:

```yaml
message:
  gateway_type: "aws_sns"
  aws_sns:
    region: "us-east-1"
    access_key_id: "YOUR_ACCESS_KEY"
    secret_access_key: "YOUR_SECRET_KEY"
```

## Configure Matrix / Palpo

1. In Palpo config (`palpo.toml`), set your server name and appservice registration directory:

```toml
server_name = "example.com"
appservice_registration_dir = "appservices"
```

2. Place your bridge registration file under that directory:
   - `appservices/message-registration.yaml`

3. Ensure tokens are consistent between Palpo registration and bridge config.

## Database Configuration

The bridge auto-detects DB type from connection string prefix:

- `postgres://` or `postgresql://` -> PostgreSQL
- `sqlite://` -> SQLite
- `mysql://` or `mariadb://` -> MySQL / MariaDB

Examples:

```yaml
database:
  url: "postgresql://user:password@localhost:5432/matrix_bridge"
  max_connections: 10
  min_connections: 1
```

```yaml
database:
  url: "sqlite://./data/matrix-bridge.db"
```

## Environment Overrides

The following environment variables are supported:

- `CONFIG_PATH` - Path to config file
- `MATRIX_BRIDGE_MESSAGE__BRIDGE__DOMAIN` - Bridge domain
- `MATRIX_BRIDGE_MESSAGE__REGISTRATION__AS_TOKEN` - AppService token
- `MATRIX_BRIDGE_MESSAGE__REGISTRATION__HS_TOKEN` - Homeserver token
- And more (see configuration documentation)

## Development

### Build

```bash
cargo build
```

### Test

```bash
cargo test
```

### Run with logging

```bash
RUST_LOG=debug cargo run
```

## License

Apache-2.0

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

## Related Projects

- [matrix-bridge-discord](https://github.com/palpo-im/matrix-bridge-discord) - Discord bridge
- [mautrix-meta](https://github.com/mautrix/meta) - Facebook/Instagram bridge (Go)

