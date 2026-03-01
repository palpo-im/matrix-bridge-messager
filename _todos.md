# Matrix Bridge Message - 开发任务清单

## 项目概述
基于 Rust 的 Matrix <-> SMS/Message 桥接器，参考 matrix-bridge-discord 架构实现。

## 开发阶段

### 阶段 1: 项目基础设施 (Phase 1: Infrastructure) ✅
- [x] 1.1 配置 Cargo.toml 依赖项
  - [x] 添加 salvo web 框架
  - [x] 添加 tokio 异步运行时
  - [x] 添加 diesel 数据库 ORM
  - [x] 添加 matrix-bot-sdk (appservice 支持)
  - [x] 添加序列化/反序列化库
  - [x] 添加日志和追踪库
  - [x] 添加配置管理库

- [x] 1.2 创建项目目录结构
  - [x] src/config/ - 配置管理
  - [x] src/db/ - 数据库模型和管理
  - [x] src/matrix/ - Matrix 客户端和事件处理
  - [x] src/message/ - SMS/Message 网关接口
  - [x] src/bridge/ - 桥接核心逻辑
  - [x] src/web/ - HTTP 端点
  - [x] src/utils/ - 工具函数
  - [x] config/ - 配置文件示例
  - [x] migrations/ - 数据库迁移

- [x] 1.3 配置文件系统
  - [x] 创建 config.sample.yaml 示例配置
  - [x] 实现配置解析器
  - [x] 实现配置验证器
  - [x] 环境变量覆盖支持

- [x] 1.4 日志和追踪系统
  - [x] 集成 tracing 日志框架
  - [x] 配置日志级别和格式
  - [x] 实现结构化日志

- [x] 1.5 错误处理框架
  - [x] 定义错误类型
  - [x] 实现 thiserror 错误转换
  - [x] 错误传播和处理策略

### 阶段 2: 数据库层 (Phase 2: Database Layer)
- [ ] 2.1 数据库连接管理
  - [ ] 实现 DatabaseManager
  - [ ] 支持多种数据库后端 (PostgreSQL, SQLite, MySQL)
  - [ ] 连接池配置
  - [ ] 数据库迁移系统

- [ ] 2.2 数据模型定义
  - [ ] User 模型 - Matrix 用户和 SMS 用户映射
  - [ ] Room 模型 - Matrix 房间和 SMS 会话映射
  - [ ] Message 模型 - 消息映射和去重
  - [ ] Portal 模型 - 桥接门户配置
  - [ ] Puppet 模型 - 虚拟用户管理

- [ ] 2.3 数据存储实现
  - [ ] UserStore - 用户数据存储
  - [ ] RoomStore - 房间数据存储
  - [ ] MessageStore - 消息数据存储
  - [ ] PortalStore - 门户数据存储

- [ ] 2.4 数据库迁移脚本
  - [ ] 创建初始迁移
  - [ ] 索引优化
  - [ ] 外键约束

### 阶段 3: Matrix 集成 (Phase 3: Matrix Integration)
- [ ] 3.1 Matrix AppService 客户端
  - [ ] 初始化 MatrixAppservice
  - [ ] 配置注册文件 (registration.yaml)
  - [ ] 实现 AS 事务处理
  - [ ] 事件过滤和路由

- [ ] 3.2 Matrix 事件处理器
  - [ ] 消息事件处理 (m.room.message)
  - [ ] 状态事件处理 (m.room.member)
  - [ ] 红action事件处理 (m.room.redaction)
  - [ ] 反应事件处理 (m.reaction)
  - [ ] 已读回执处理 (m.read)
  - [ ] 正在输入处理 (m.typing)

- [ ] 3.3 Matrix 消息发送
  - [ ] 文本消息发送
  - [ ] 媒体消息发送 (图片、视频、文件)
  - [ ] 消息编辑
  - [ ] 消息删除
  - [ ] 反应发送

- [ ] 3.4 Matrix 用户管理
  - [ ] 虚拟用户创建
  - [ ] 用户信息同步 (头像、昵称)
  - [ ] 用户权限管理
  - [ ] 双击用户映射

- [ ] 3.5 Matrix 房间管理
  - [ ] 房间创建
  - [ ] 房间邀请处理
  - [ ] 房间成员管理
  - [ ] 房间元数据同步

### 阶段 4: SMS/Message 网关集成 (Phase 4: Message Gateway)
- [ ] 4.1 SMS 网关抽象接口
  - [ ] 定义 MessageGateway trait
  - [ ] 定义消息类型和状态
  - [ ] 定义联系人和会话模型

- [ ] 4.2 SMS 网关实现 (可插拔后端)
  - [ ] Twilio API 集成
  - [ ] AWS SNS 集成
  - [ ] 本地 GSM 调制解调器支持
  - [ ] 模拟网关 (用于测试)

- [ ] 4.3 消息发送
  - [ ] 发送 SMS 文本消息
  - [ ] 发送 MMS 媒体消息
  - [ ] 消息状态跟踪
  - [ ] 发送失败重试机制

- [ ] 4.4 消息接收
  - [ ] Webhook 接收器 (Twilio/SNS)
  - [ ] 消息去重
  - [ ] 消息解析和验证
  - [ ] 错误处理

- [ ] 4.5 联系人管理
  - [ ] 联系人信息获取
  - [ ] 联系人缓存
  - [ ] 联系人同步

### 阶段 5: 桥接核心逻辑 (Phase 5: Bridge Core)
- [ ] 5.1 BridgeCore 实现
  - [ ] 桥接器初始化
  - [ ] 消息队列管理
  - [ ] 事件循环
  - [ ] 优雅关闭

- [ ] 5.2 消息流转
  - [ ] Matrix -> SMS 消息转换
  - [ ] SMS -> Matrix 消息转换
  - [ ] 消息格式转换
  - [ ] 媒体文件处理

- [ ] 5.3 门户管理
  - [ ] 自动创建门户房间
  - [ ] 门户状态同步
  - [ ] 门户配置管理
  - [ ] 门户删除和清理

- [ ] 5.4 用户同步
  - [ ] Matrix 用户到 SMS 联系人映射
  - [ ] SMS 联系人到 Matrix 虚拟用户映射
  - [ ] 用户状态同步 (在线、离线)
  - [ ] 用户元数据同步 (头像、昵称)

- [ ] 5.5 状态同步
  - [ ] 已读回执同步
  - [ ] 正在输入状态同步
  - [ ] 消息状态同步 (发送中、已发送、已送达、已读)
  - [ ] 在线状态同步

- [ ] 5.6 媒体处理
  - [ ] 媒体下载和缓存
  - [ ] 媒体格式转换
  - [ ] 媒体上传到 Matrix
  - [ ] 媒体上传到 SMS 网关

### 阶段 6: Web 服务和管理接口 (Phase 6: Web Service & Admin)
- [ ] 6.1 HTTP 服务器
  - [ ] Salvo 服务器初始化
  - [ ] 路由配置
  - [ ] 中间件配置 (CORS、日志等)

- [ ] 6.2 健康检查端点
  - [ ] /health - 健康检查
  - [ ] /ready - 就绪检查
  - [ ] /status - 状态详情

- [ ] 6.3 指标端点
  - [ ] Prometheus 指标导出
  - [ ] 自定义指标 (消息数、延迟等)

- [ ] 6.4 Provisioning API
  - [ ] 用户登录/注销
  - [ ] 门户管理接口
  - [ ] 桥接状态查询
  - [ ] 管理操作接口

- [ ] 6.5 Webhook 接收器
  - [ ] SMS 网关 webhook 处理
  - [ ] 签名验证
  - [ ] 请求验证

### 阶段 7: 命令行和管理工具 (Phase 7: CLI & Admin)
- [ ] 7.1 命令行参数解析
  - [ ] 配置文件路径
  - [ ] 日志级别设置
  - [ ] 运行模式 (生产/开发)

- [ ] 7.2 Matrix 管理命令
  - [ ] !message help - 帮助命令
  - [ ] !message login - 登录 SMS 网关
  - [ ] !message logout - 注销
  - [ ] !message ping - 测试连接
  - [ ] !message bridge - 手动创建桥接
  - [ ] !message unbridge - 删除桥接

- [ ] 7.3 管理员命令
  - [ ] !message admin clean-cache - 清理缓存
  - [ ] !message admin stats - 统计信息
  - [ ] !message admin users - 用户列表

### 阶段 8: 测试和质量保证 (Phase 8: Testing & QA)
- [ ] 8.1 单元测试
  - [ ] 配置解析测试
  - [ ] 消息转换测试
  - [ ] 数据库操作测试
  - [ ] 工具函数测试

- [ ] 8.2 集成测试
  - [ ] Matrix 客户端集成测试
  - [ ] SMS 网关集成测试 (模拟)
  - [ ] 桥接逻辑集成测试
  - [ ] Web API 集成测试

- [ ] 8.3 端到端测试
  - [ ] 消息双向流转测试
  - [ ] 媒体消息测试
  - [ ] 状态同步测试
  - [ ] 错误场景测试

- [ ] 8.4 性能测试
  - [ ] 消息吞吐量测试
  - [ ] 并发连接测试
  - [ ] 内存使用测试
  - [ ] 数据库性能测试

### 阶段 9: 文档和部署 (Phase 9: Documentation & Deployment)
- [ ] 9.1 用户文档
  - [ ] README.md - 项目介绍
  - [ ] README_CN.md - 中文介绍
  - [ ] 安装指南
  - [ ] 配置指南
  - [ ] 使用指南

- [ ] 9.2 开发者文档
  - [ ] 架构设计文档
  - [ ] API 文档
  - [ ] 数据库 Schema 文档
  - [ ] 贡献指南 (CONTRIBUTING.md)

- [ ] 9.3 部署配置
  - [ ] Dockerfile
  - [ ] docker-compose.yml
  - [ ] Kubernetes 部署配置
  - [ ] systemd 服务文件

- [ ] 9.4 CI/CD
  - [ ] GitHub Actions 工作流
  - [ ] 自动化测试
  - [ ] 自动化构建
  - [ ] 自动化发布

### 阶段 10: 优化和增强 (Phase 10: Optimization & Enhancements)
- [ ] 10.1 性能优化
  - [ ] 消息队列优化
  - [ ] 数据库查询优化
  - [ ] 缓存策略优化
  - [ ] 并发处理优化

- [ ] 10.2 可靠性增强
  - [ ] 消息持久化和恢复
  - [ ] 断线重连机制
  - [ ] 错误重试策略
  - [ ] 降级处理

- [ ] 10.3 安全增强
  - [ ] 输入验证和清理
  - [ ] SQL 注入防护
  - [ ] XSS 防护
  - [ ] Webhook 签名验证
  - [ ] 敏感信息加密存储

- [ ] 10.4 监控和告警
  - [ ] 结构化日志
  - [ ] 分布式追踪
  - [ ] 告警规则配置
  - [ ] 监控仪表板

## 参考项目
- matrix-bridge-discord (Rust) - 主要架构参考
- mautrix-meta (Go) - 功能特性参考
- SmsMatrix (Java) - SMS bridge 实现参考

## 技术栈
- 语言: Rust (Edition 2024)
- Web 框架: Salvo
- 异步运行时: Tokio
- 数据库 ORM: Diesel (PostgreSQL/SQLite/MySQL)
- Matrix SDK: matrix-bot-sdk
- 日志: tracing
- 配置: config-rs
- 序列化: serde

## 开发优先级
1. **P0 - 核心功能**: 阶段 1-5 (基础设施到桥接核心)
2. **P1 - 必需功能**: 阶段 6-7 (Web 服务和管理工具)
3. **P2 - 质量保证**: 阶段 8 (测试)
4. **P3 - 生产就绪**: 阶段 9 (文档和部署)
5. **P4 - 增强**: 阶段 10 (优化和增强)
