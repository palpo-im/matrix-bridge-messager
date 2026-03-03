# CONTRIBUTING.md

## 贡献指南

感谢您考虑为 Matrix Bridge Message 项目做出贡献！

## 开发环境设置

### 前置要求

- Rust 1.93 或更高版本
- PostgreSQL / SQLite / MySQL（可选，用于测试）
- Git

### 设置步骤

1. **克隆仓库**
```bash
git clone https://github.com/palpo-im/matrix-bridge-messager.git
cd matrix-bridge-messager
```

2. **安装依赖**
```bash
cargo build
```

3. **运行测试**
```bash
cargo test
```

4. **配置开发环境**
```bash
cp config/config.sample.yaml config.yaml
# 编辑 config.yaml，设置测试配置
```

## 开发流程

### 1. 创建分支

```bash
git checkout -b feature/your-feature-name
```

### 2. 编写代码

- 遵循 Rust 代码规范
- 添加适当的注释和文档
- 编写单元测试

### 3. 运行检查

```bash
# 格式化代码
cargo fmt

# 检查代码
cargo clippy

# 运行测试
cargo test
```

### 4. 提交更改

```bash
git add .
git commit -m "feat: 简短描述你的更改"
```

提交消息格式：
- `feat:` 新功能
- `fix:` 错误修复
- `docs:` 文档更新
- `test:` 测试相关
- `refactor:` 代码重构
- `chore:` 构建/工具链相关

### 5. 推送并创建 PR

```bash
git push origin feature/your-feature-name
```

然后在 GitHub 上创建 Pull Request。

## 代码规范

### Rust 代码风格

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### 文档

- 所有公共 API 必须有文档注释
- 使用 `///` 进行文档注释
- 包含示例代码

```rust
/// 发送 SMS 消息
/// 
/// # Arguments
/// 
/// * `to` - 目标电话号码
/// * `body` - 消息内容
/// 
/// # Example
/// 
/// ```
/// let gateway = MockGateway::new(true);
/// gateway.send_message("+1234567890", "Hello").await?;
/// ```
pub async fn send_message(&self, to: &str, body: &str) -> Result<String>;
```

### 错误处理

- 使用 `anyhow::Result` 进行错误传播
- 使用 `thiserror` 定义自定义错误类型
- 提供有意义的错误消息

### 测试

- 为所有公共函数编写单元测试
- 使用 `#[tokio::test]` 进行异步测试
- 使用 `test-case` crate 进行参数化测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }
}
```

## 架构

### 项目结构

```
src/
├── config/          # 配置管理
├── db/              # 数据库层
├── matrix/          # Matrix 集成
├── message/         # SMS 网关
├── bridge/          # 桥接核心
├── web/             # HTTP 服务
└── utils/           # 工具函数
```

### 关键组件

1. **Config**: 配置管理和验证
2. **DatabaseManager**: 数据库连接和迁移
3. **MatrixAppservice**: Matrix 应用服务
4. **MessageGateway**: SMS 网关抽象
5. **BridgeCore**: 消息桥接逻辑
6. **WebServer**: HTTP 端点

## 发布流程

1. 更新 `Cargo.toml` 中的版本号
2. 更新 `CHANGELOG.md`
3. 创建 git tag
4. 构建 Docker 镜像
5. 发布到 GitHub Releases

## 获取帮助

- GitHub Issues: https://github.com/palpo-im/matrix-bridge-messager/issues
- Matrix Room: #message-bridge:palpo.im
- Email: chris@acroidea.com

## 许可证

通过提交代码，您同意您的贡献将根据 Apache-2.0 许可证进行许可。

