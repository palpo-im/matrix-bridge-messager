# Configuration Examples

This file contains various configuration examples for different use cases.

## Table of Contents

1. [Basic Development Configuration](#basic-development-configuration)
2. [Production Configuration with PostgreSQL](#production-configuration-with-postgresql)
3. [Production Configuration with Twilio](#production-configuration-with-twilio)
4. [Docker Configuration](#docker-configuration)
5. [Testing Configuration](#testing-configuration)

## Basic Development Configuration

Simple configuration for local development with SQLite and Mock gateway.

```yaml
bridge:
  domain: "localhost"
  homeserver_url: "http://localhost:8008"
  bind_address: "127.0.0.1"
  port: 9006
  bot_username: "message-bot"

registration:
  id: "message"
  url: "http://localhost:9006"
  as_token: "dev_as_token_change_in_production"
  hs_token: "dev_hs_token_change_in_production"
  sender_localpart: "_message_"

message:
  gateway_type: "mock"
  mock:
    enabled: true

database:
  url: "sqlite://./data/bridge.db"

logging:
  level: "debug"
  format: "pretty"

behavior:
  auto_create_portals: true
  sync_contacts: true
  enable_read_receipts: true
  enable_typing_notifications: true
  max_message_age: 86400

limits:
  max_file_size: 104857600
  matrix_event_age_limit_ms: 300000
  message_rate_limit: 100

admin_users:
  - "@admin:localhost"
```

## Production Configuration with PostgreSQL

Production-ready configuration with PostgreSQL backend.

```yaml
bridge:
  domain: "example.com"
  homeserver_url: "https://matrix.example.com"
  bind_address: "0.0.0.0"
  port: 9006
  bot_username: "message-bot"

registration:
  id: "message"
  url: "https://bridge.example.com"
  as_token: "${AS_TOKEN}"
  hs_token: "${HS_TOKEN}"
  sender_localpart: "_message_"

message:
  gateway_type: "twilio"
  twilio:
    account_sid: "${TWILIO_ACCOUNT_SID}"
    auth_token: "${TWILIO_AUTH_TOKEN}"
    phone_number: "${TWILIO_PHONE_NUMBER}"

database:
  url: "postgresql://bridge_user:${DB_PASSWORD}@db.example.com:5432/bridge_db"
  max_connections: 20
  min_connections: 5

logging:
  level: "info"
  format: "json"

behavior:
  auto_create_portals: true
  sync_contacts: true
  enable_read_receipts: true
  enable_typing_notifications: true
  max_message_age: 86400

limits:
  max_file_size: 52428800
  matrix_event_age_limit_ms: 300000
  message_rate_limit: 50

admin_users:
  - "@admin:example.com"
  - "@ops:example.com"
```

## Production Configuration with Twilio

Full Twilio configuration with all options.

```yaml
bridge:
  domain: "company.com"
  homeserver_url: "https://matrix.company.com"
  bind_address: "0.0.0.0"
  port: 9006
  bot_username: "sms-bot"

registration:
  id: "message"
  url: "https://sms-bridge.company.com"
  as_token: "${MATRIX_AS_TOKEN}"
  hs_token: "${MATRIX_HS_TOKEN}"
  sender_localpart: "_sms_"

message:
  gateway_type: "twilio"
  twilio:
    account_sid: "${TWILIO_ACCOUNT_SID}"
    auth_token: "${TWILIO_AUTH_TOKEN}"
    phone_number: "+1234567890"

database:
  url: "postgresql://sms_bridge:${DB_PASSWORD}@postgres.company.internal:5432/sms_bridge"
  max_connections: 15
  min_connections: 3

logging:
  level: "info"
  format: "json"

behavior:
  auto_create_portals: true
  sync_contacts: true
  enable_read_receipts: true
  enable_typing_notifications: false  # Disable to reduce API calls
  max_message_age: 43200  # 12 hours

limits:
  max_file_size: 10485760  # 10MB
  matrix_event_age_limit_ms: 600000  # 10 minutes
  message_rate_limit: 30  # 30 messages per minute

admin_users:
  - "@admin:company.com"
```

## Docker Configuration

Configuration for Docker deployment with environment variables.

```yaml
bridge:
  domain: "${MATRIX_DOMAIN}"
  homeserver_url: "${MATRIX_HOMESERVER_URL}"
  bind_address: "0.0.0.0"
  port: 9006
  bot_username: "message-bot"

registration:
  id: "${BRIDGE_ID}"
  url: "${BRIDGE_URL}"
  as_token: "${AS_TOKEN}"
  hs_token: "${HS_TOKEN}"
  sender_localpart: "${SENDER_LOCALPART}"

message:
  gateway_type: "${GATEWAY_TYPE}"
  twilio:
    account_sid: "${TWILIO_ACCOUNT_SID}"
    auth_token: "${TWILIO_AUTH_TOKEN}"
    phone_number: "${TWILIO_PHONE_NUMBER}"
  mock:
    enabled: "${MOCK_ENABLED:-false}"

database:
  url: "${DATABASE_URL}"
  max_connections: ${DB_MAX_CONNECTIONS:-10}
  min_connections: ${DB_MIN_CONNECTIONS:-1}

logging:
  level: "${LOG_LEVEL:-info}"
  format: "${LOG_FORMAT:-json}"

behavior:
  auto_create_portals: ${AUTO_CREATE_PORTALS:-true}
  sync_contacts: ${SYNC_CONTACTS:-true}
  enable_read_receipts: ${ENABLE_READ_RECEIPTS:-true}
  enable_typing_notifications: ${ENABLE_TYPING:-true}
  max_message_age: ${MAX_MESSAGE_AGE:-86400}

limits:
  max_file_size: ${MAX_FILE_SIZE:-104857600}
  matrix_event_age_limit_ms: ${EVENT_AGE_LIMIT:-300000}
  message_rate_limit: ${RATE_LIMIT:-100}

admin_users: ${ADMIN_USERS:-[]}
```

**docker-compose.yml**:
```yaml
version: '3.8'

services:
  bridge:
    image: ghcr.io/palpo-im/matrix-bridge-messager:latest
    container_name: matrix-bridge-messager
    restart: unless-stopped
    ports:
      - "9006:9006"
    environment:
      - MATRIX_DOMAIN=example.com
      - MATRIX_HOMESERVER_URL=https://matrix.example.com
      - BRIDGE_ID=message
      - BRIDGE_URL=https://bridge.example.com
      - AS_TOKEN=${AS_TOKEN}
      - HS_TOKEN=${HS_TOKEN}
      - SENDER_LOCALPART=_message_
      - GATEWAY_TYPE=twilio
      - TWILIO_ACCOUNT_SID=${TWILIO_ACCOUNT_SID}
      - TWILIO_AUTH_TOKEN=${TWILIO_AUTH_TOKEN}
      - TWILIO_PHONE_NUMBER=${TWILIO_PHONE_NUMBER}
      - DATABASE_URL=postgresql://bridge:password@db:5432/bridge
      - LOG_LEVEL=info
      - LOG_FORMAT=json
      - ADMIN_USERS=["@admin:example.com"]
    depends_on:
      - db
    networks:
      - bridge-network

  db:
    image: postgres:15-alpine
    container_name: matrix-bridge-messager-db
    restart: unless-stopped
    environment:
      - POSTGRES_USER=bridge
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=bridge
    volumes:
      - postgres-data:/var/lib/postgresql/data
    networks:
      - bridge-network

volumes:
  postgres-data:

networks:
  bridge-network:
    driver: bridge
```

## Testing Configuration

Configuration for running tests.

```yaml
bridge:
  domain: "test.local"
  homeserver_url: "http://localhost:8008"
  bind_address: "127.0.0.1"
  port: 19006
  bot_username: "test-bot"

registration:
  id: "test-message"
  url: "http://localhost:19006"
  as_token: "test_as_token"
  hs_token: "test_hs_token"
  sender_localpart: "_test_message_"

message:
  gateway_type: "mock"
  mock:
    enabled: true

database:
  url: "sqlite://./test/test.db"

logging:
  level: "debug"
  format: "pretty"

behavior:
  auto_create_portals: true
  sync_contacts: true
  enable_read_receipts: true
  enable_typing_notifications: true
  max_message_age: 86400

limits:
  max_file_size: 104857600
  matrix_event_age_limit_ms: 300000
  message_rate_limit: 1000

admin_users:
  - "@test:test.local"
```

## Environment Variables

All configuration options can be overridden with environment variables:

```bash
# Bridge settings
export MATRIX_BRIDGE_MESSAGER__BRIDGE__DOMAIN="example.com"
export MATRIX_BRIDGE_MESSAGER__BRIDGE__HOMESERVER_URL="https://matrix.example.com"
export MATRIX_BRIDGE_MESSAGER__BRIDGE__BIND_ADDRESS="0.0.0.0"
export MATRIX_BRIDGE_MESSAGER__BRIDGE__PORT="9006"

# Registration settings
export MATRIX_BRIDGE_MESSAGER__REGISTRATION__ID="message"
export MATRIX_BRIDGE_MESSAGER__REGISTRATION__AS_TOKEN="your_token"
export MATRIX_BRIDGE_MESSAGER__REGISTRATION__HS_TOKEN="your_token"

# Database settings
export MATRIX_BRIDGE_MESSAGER__DATABASE__URL="postgresql://user:pass@host/db"
export MATRIX_BRIDGE_MESSAGER__DATABASE__MAX_CONNECTIONS="20"

# Message gateway settings
export MATRIX_BRIDGE_MESSAGER__MESSAGE__GATEWAY_TYPE="twilio"
export MATRIX_BRIDGE_MESSAGER__MESSAGE__TWILIO__ACCOUNT_SID="your_sid"
export MATRIX_BRIDGE_MESSAGER__MESSAGE__TWILIO__AUTH_TOKEN="your_token"
export MATRIX_BRIDGE_MESSAGER__MESSAGE__TWILIO__PHONE_NUMBER="+1234567890"

# Logging settings
export MATRIX_BRIDGE_MESSAGER__LOGGING__LEVEL="info"
export MATRIX_BRIDGE_MESSAGER__LOGGING__FORMAT="json"

# Special environment variables
export CONFIG_PATH="/path/to/config.yaml"
export RUST_LOG="debug"
```

## Configuration Validation

Validate your configuration before running:

```bash
cargo run -- validate-config
```

This will check:
- Required fields are present
- Values are valid
- Tokens are not placeholder values
- Database URL is valid
- Gateway type is supported


