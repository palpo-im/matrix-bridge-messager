# 架构设计文档

## 概述

Matrix Bridge Messager 是一个用 Rust 编写的 Matrix <-> SMS/消息桥接器，采用模块化设计，支持多种数据库后端和 SMS 网关。

## 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                     Matrix Bridge Messager                    │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐      ┌──────────────┐      ┌────────────┐ │
│  │   CLI/主程序  │─────▶│  Bridge Core │◀─────│ Web Server │ │
│  └──────────────┘      └───────┬──────┘      └────────────┘ │
│                                │                              │
│                 ┌──────────────┼──────────────┐              │
│                 │              │              │              │
│        ┌────────▼──────┐ ┌────▼────────┐ ┌──▼──────────┐    │
│        │ Matrix Client │ │ SMS Gateway │ │     DB      │    │
│        │  (AppService) │ │  (Twilio)   │ │ (PostgreSQL)│    │
│        └───────────────┘ └─────────────┘ └─────────────┘    │
│                                                               │
└─────────────────────────────────────────────────────────────┘
         │                      │                    │
         ▼                      ▼                    ▼
    Matrix HS              Twilio API          Database
```

## 核心组件

### 1. 配置管理 (Config)

**位置**: `src/config/`

**职责**:
- YAML 配置文件解析
- 环境变量覆盖
- 配置验证
- 运行时配置访问

**关键结构**:
```rust
pub struct Config {
    pub bridge: BridgeConfig,
    pub registration: RegistrationConfig,
    pub message: MessageConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub behavior: BehaviorConfig,
    pub limits: LimitsConfig,
    pub admin_users: Vec<String>,
}
```

### 2. 数据库层 (Database)

**位置**: `src/db/`

**职责**:
- 多数据库后端支持 (PostgreSQL, SQLite, MySQL)
- 数据模型定义
- CRUD 操作
- 数据库迁移

**关键组件**:
- `DatabaseManager`: 数据库连接管理
- `UserStore`: 用户映射存储
- `RoomStore`: 房间映射存储
- `MessageStore`: 消息映射存储
- `EventStore`: 事件去重
- `PortalStore`: 门户配置存储

**数据模型**:
```
UserMapping: Matrix 用户 <-> SMS 联系人
RoomMapping: Matrix 房间 <-> SMS 会话
MessageMapping: Matrix 事件 <-> SMS 消息
ProcessedEvent: 事件去重
PortalConfig: 门户配置
```

### 3. Matrix 集成 (Matrix)

**位置**: `src/matrix/`

**职责**:
- Matrix AppService 协议实现
- 事件接收和处理
- Matrix API 调用
- 虚拟用户管理

**关键组件**:
- `MatrixAppservice`: AppService 客户端
- `MatrixEventHandler`: 事件处理器
- `MatrixEventProcessor`: 事件处理器（带过滤）
- `MatrixCommandHandler`: 命令解析器

**事件流程**:
```
Matrix HS -> AppService -> EventProcessor -> EventHandler -> BridgeCore
```

### 4. SMS 网关 (Message Gateway)

**位置**: `src/message/`

**职责**:
- SMS 网关抽象
- Twilio API 集成
- 消息发送和接收
- 联系人管理

**关键接口**:
```rust
#[async_trait]
pub trait MessageGateway: Send + Sync {
    async fn send_message(&self, to: &str, body: &str) -> Result<String>;
    async fn get_contact_name(&self, phone_number: &str) -> Result<Option<String>>;
    async fn health_check(&self) -> Result<bool>;
}
```

**实现**:
- `MockGateway`: 测试用模拟网关
- `TwilioGateway`: Twilio API 集成

### 5. 桥接核心 (Bridge Core)

**位置**: `src/bridge/`

**职责**:
- 消息路由
- 用户映射
- 门户管理
- 状态同步

**消息流程**:
```
Matrix -> SMS:
  Matrix Event -> BridgeCore -> MessageGateway -> SMS

SMS -> Matrix:
  SMS Message -> BridgeCore -> MatrixAppservice -> Matrix Room
```

### 6. Web 服务 (Web Server)

**位置**: `src/web/`

**职责**:
- HTTP 端点
- 健康检查
- 状态查询
- Webhook 接收

**端点**:
- `GET /health` - 健康检查
- `GET /ready` - 就绪检查
- `GET /status` - 状态详情

### 7. 命令行接口 (CLI)

**位置**: `src/cli.rs`

**职责**:
- 命令行参数解析
- 管理命令执行
- 注册文件生成

**命令**:
- `generate-registration` - 生成注册文件
- `validate-config` - 验证配置
- `test-gateway` - 测试网关
- `list-portals` - 列出门户
- `unbridge` - 删除桥接

## 数据流

### Matrix -> SMS 消息流

```
1. 用户在 Matrix 房间发送消息
2. Matrix HS 通过 AppService 事务推送事件
3. BridgeAppserviceHandler 接收事务
4. MatrixEventProcessor 过滤和处理事件
5. MatrixEventHandler 调用 BridgeCore
6. BridgeCore 查询 RoomMapping
7. BridgeCore 调用 MessageGateway.send_message()
8. SMS 网关发送消息
9. BridgeCore 创建 MessageMapping 记录
```

### SMS -> Matrix 消息流

```
1. SMS 网关接收消息（通过 Webhook 或轮询）
2. Web Server 接收 Webhook 请求
3. BridgeCore.handle_incoming_message()
4. BridgeCore 查询 RoomMapping
5. 如果房间不存在，创建新房间
6. MatrixAppservice.send_message()
7. Matrix HS 发送消息到房间
8. BridgeCore 创建 MessageMapping 记录
```

## 配置层次

```yaml
config.yaml (文件)
    ↓
环境变量覆盖 (MATRIX_BRIDGE_MESSAGER__*)
    ↓
Config 结构体
    ↓
各模块访问
```

## 错误处理

```
Error Types:
  - anyhow::Error: 通用错误
  - thiserror: 自定义错误类型
  - DatabaseError: 数据库错误
  - Validation errors: 配置验证错误

Error Flow:
  1. 底层错误 (IO, Network, DB)
  2. 转换为应用错误
  3. 错误传播 (Result<T>)
  4. 顶层错误处理和日志
```

## 安全考虑

1. **认证**:
   - Matrix AppService tokens
   - SMS 网关认证
   - Webhook 签名验证

2. **授权**:
   - 管理员权限检查
   - 房间权限验证
   - 命令权限控制

3. **数据安全**:
   - 敏感信息加密存储
   - SQL 注入防护
   - 输入验证和清理

## 性能优化

1. **数据库**:
   - 连接池
   - 索引优化
   - 批量操作

2. **消息处理**:
   - 异步处理
   - 消息队列
   - 事件去重

3. **缓存**:
   - 用户映射缓存
   - 房间映射缓存
   - 联系人信息缓存

## 扩展性

### 添加新的 SMS 网关

1. 实现 `MessageGateway` trait
2. 在 `create_gateway()` 中添加分支
3. 更新配置结构

### 添加新的 Matrix 事件类型

1. 在 `MatrixEventHandler` 中添加方法
2. 在 `MatrixEventProcessor` 中添加匹配分支
3. 在 `BridgeCore` 中添加处理逻辑

### 添加新的数据库后端

1. 创建新的 Store 实现
2. 在 `DatabaseManager::new()` 中添加分支
3. 添加数据库迁移脚本

## 部署架构

```
┌─────────────────────────────────────────┐
│         Load Balancer (Optional)         │
└────────────────┬────────────────────────┘
                 │
    ┌────────────┼────────────┐
    │            │            │
┌───▼───┐    ┌───▼───┐    ┌───▼───┐
│ Bridge│    │ Bridge│    │ Bridge│
│  #1   │    │  #2   │    │  #3   │
└───┬───┘    └───┬───┘    └───┬───┘
    │            │            │
    └────────────┼────────────┘
                 │
         ┌───────▼────────┐
         │   Database     │
         │  (PostgreSQL)  │
         └────────────────┘
```

## 监控和日志

1. **日志**:
   - 结构化日志 (tracing)
   - 日志级别配置
   - JSON 格式支持

2. **指标** (计划):
   - Prometheus 指标
   - 消息延迟
   - 错误率
   - 吞吐量

3. **健康检查**:
   - HTTP 端点
   - 数据库连接
   - 网关连接

## 故障恢复

1. **断线重连**:
   - Matrix 连接重试
   - 数据库连接池
   - HTTP 客户端重试

2. **消息持久化**:
   - 数据库存储
   - 事件日志
   - 消息去重

3. **优雅关闭**:
   - 信号处理
   - 任务取消
   - 资源清理

## 开发指南

### 本地开发

```bash
# 1. 克隆仓库
git clone <repo>
cd matrix-bridge-messager

# 2. 配置
cp config/config.sample.yaml config.yaml
# 编辑 config.yaml

# 3. 运行
cargo run
```

### 测试

```bash
# 单元测试
cargo test

# 集成测试
cargo test --test integration_tests

# 覆盖率
cargo tarpaulin
```

### 构建

```bash
# Debug
cargo build

# Release
cargo build --release

# Docker
docker build -t matrix-bridge-messager .
```

## 未来改进

1. **功能增强**:
   - 媒体消息支持
   - 群组桥接
   - 端到端加密

2. **性能优化**:
   - 消息批处理
   - 缓存优化
   - 连接池调优

3. **可靠性**:
   - 消息确认机制
   - 重试策略
   - 备份和恢复

4. **可观测性**:
   - 分布式追踪
   - 告警系统
   - 性能监控


