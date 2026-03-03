# matrix-bridge-messager

用 Rust 编写的 Matrix <-> SMS/消息桥接器。

维护者：`Palpo Team`  
联系方式：`chris@acroidea.com`

## 概述

- 纯 Rust 实现的 Matrix <-> SMS/消息桥接
- Matrix 应用服务 + SMS 网关桥接核心
- HTTP 端点用于健康检查/状态/指标和配置
- 数据库后端：PostgreSQL、SQLite 和 MySQL（功能门控）
- Dockerfile 用于本地构建和容器运行

## 状态

🚧 **积极开发中** - 本项目处于早期开发阶段。

## 功能特性（计划中）

- [x] 项目结构和配置
- [x] Matrix AppService 集成（框架）
- [x] SMS 网关支持（Mock, Twilio）
- [x] 双向消息桥接（框架）
- [ ] 媒体消息支持
- [ ] 已读回执和输入通知
- [ ] 联系人同步
- [ ] 多数据库支持

## 仓库结构

- `src/`：桥接器实现
- `config/config.sample.yaml`：示例配置
- `migrations/`：数据库迁移
- `Dockerfile`：多阶段容器构建

## 前置要求

- Rust 工具链（与项目兼容；Docker 构建使用 Rust 1.93）
- 配置了应用服务的 Matrix 主服务器
- SMS 网关凭证（Twilio）
- 数据库：PostgreSQL、SQLite 或 MySQL

## 快速开始（本地）

1. 创建配置文件：

```bash
cp config/config.sample.yaml config.yaml
```

2. 在 `config.yaml` 中设置必需的值：
   - `bridge.domain`
   - `bridge.homeserver_url`
   - `database.url`
   - `message.gateway_type` 和对应的网关凭证
   - 注册值（`as_token`、`hs_token`）

3. 运行：

```bash
cargo check
cargo run
```

## 配置 SMS 网关

### Twilio

1. 在 https://www.twilio.com 注册
2. 从控制台获取你的 Account SID 和 Auth Token
3. 购买或使用 Twilio 电话号码
4. 在 `config.yaml` 中配置：

```yaml
message:
  gateway_type: "twilio"
  twilio:
    account_sid: "YOUR_ACCOUNT_SID"
    auth_token: "YOUR_AUTH_TOKEN"
    phone_number: "+1234567890"
```

## 配置 Matrix / Palpo

1. 在 Palpo 配置（`palpo.toml`）中，设置服务器名称和应用服务注册目录：

```toml
server_name = "example.com"
appservice_registration_dir = "appservices"
```

2. 将桥接器注册文件放在该目录下：
   - `appservices/message-registration.yaml`

3. 确保 Palpo 注册和桥接器配置之间的令牌一致。

## 数据库配置

桥接器从连接字符串前缀自动检测数据库类型：

- `postgres://` 或 `postgresql://` -> PostgreSQL
- `sqlite://` -> SQLite
- `mysql://` 或 `mariadb://` -> MySQL / MariaDB

示例：

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

## 环境变量覆盖

支持以下环境变量：

- `CONFIG_PATH` - 配置文件路径
- `MATRIX_BRIDGE_MESSAGE__BRIDGE__DOMAIN` - 桥接器域
- `MATRIX_BRIDGE_MESSAGE__REGISTRATION__AS_TOKEN` - AppService 令牌
- `MATRIX_BRIDGE_MESSAGE__REGISTRATION__HS_TOKEN` - 主服务器令牌
- 以及更多（参见配置文档）

## 开发

### 构建

```bash
cargo build
```

### 测试

```bash
cargo test
```

### 带日志运行

```bash
RUST_LOG=debug cargo run
```

## Docker

构建：

```bash
docker build -t ghcr.io/palpo-im/matrix-bridge-messager:main -f Dockerfile .
```

运行（需要在挂载目录中有 `/data/config.yaml`）：

```bash
docker run --rm \
  -p 9006:9006 \
  -v "$(pwd)/config:/data" \
  -e CONFIG_PATH=/data/config.yaml \
  ghcr.io/palpo-im/matrix-bridge-messager:main
```

或使用 docker-compose：

```bash
docker-compose up -d
```

## 命令行工具

```bash
# 生成注册文件
cargo run -- generate-registration -o message-registration.yaml

# 验证配置
cargo run -- validate-config

# 测试网关连接
cargo run -- test-gateway -t +1234567890 -m "Test message"

# 查看帮助
cargo run -- --help
```

## 许可证

Apache-2.0

## 贡献

欢迎贡献！请在提交 PR 之前阅读我们的贡献指南。

## 相关项目

- [matrix-bridge-discord](https://github.com/palpo-im/matrix-bridge-discord) - Discord 桥接器
- [mautrix-meta](https://github.com/mautrix/meta) - Facebook/Instagram 桥接器（Go）

