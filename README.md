# AuthCore 🔐

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-13+-blue.svg)](https://www.postgresql.org/)
[![Redis](https://img.shields.io/badge/Redis-6+-red.svg)](https://redis.io/)

一个基于 Rust 构建的高性能身份认证和用户管理系统，提供完整的 JWT 认证、用户管理、文件上传等功能。

- 慧添翼架构设计：https://huiwings.cn/arch
- 慧添翼小程序展示：https://huiwings.cn/show

## 📋 目录

- [项目简介](#项目简介)
- [功能特性](#功能特性)
- [技术栈](#技术栈)
- [快速开始](#快速开始)
- [项目结构](#项目结构)
- [API 文档](#api-文档)
- [配置说明](#配置说明)
- [开发指南](#开发指南)
- [贡献指南](#贡献指南)
- [许可证](#许可证)

## 🎯 项目简介

AuthCore 是从内部项目提取并开源的身份认证核心系统，专注于提供安全、高性能的用户认证和管理服务。项目采用微服务架构，支持高并发访问，适用于各种规模的应用程序。

### 主要特点

- 🔐 **安全认证**: 基于 JWT 的令牌认证系统
- 🚀 **高性能**: Rust 语言构建，内存安全且性能优异
- 🏗️ **微服务架构**: 模块化设计，易于扩展和维护
- 🔧 **开发友好**: 完善的工具链和开发体验
- 📦 **开箱即用**: 提供完整的用户管理功能

## ✨ 功能特性

### 核心功能
- **用户认证**: JWT 令牌生成、验证和管理
- **用户管理**: 用户注册、登录、信息管理
- **权限控制**: 基于角色的访问控制
- **会话管理**: Redis 缓存支持的用户会话
- **安全加密**: AES 加密和证书管理

### 扩展功能
- **微信集成**: 微信小程序和公众号支持
- **文件上传**: 又拍云文件存储集成
- **数据库管理**: PostgreSQL 数据库支持
- **日志系统**: 完整的日志记录和监控
- **测试框架**: 内置测试工具和脚手架

## 🛠️ 技术栈

### 后端技术
- **语言**: Rust (Edition 2021)
- **Web 框架**: Axum
- **数据库**: PostgreSQL
- **ORM**: Diesel
- **缓存**: Redis
- **认证**: JWT (jsonwebtoken)
- **加密**: AES, Ring, Rust-crypto
- **序列化**: Serde
- **日志**: Log4rs, Tracing
- **HTTP 客户端**: Reqwest
- **时间处理**: Chrono, Time

### 开发工具
- **包管理**: Cargo
- **测试**: Rust 内置测试框架
- **代码格式化**: rustfmt
- **依赖管理**: Workspace 模式
- **自动化**: Dependabot

## 🚀 快速开始

### 环境要求

- Rust 1.70+
- PostgreSQL 13+
- Redis 6+
- Docker (可选)

### 安装步骤

1. **克隆项目**
```bash
git clone https://github.com/your-org/AuthCore.git
cd AuthCore
```

2. **安装依赖**
```bash
cargo build
```

3. **配置环境变量**
```bash
cp .env.example .env
# 编辑 .env 文件，配置数据库连接等信息
```

4. **启动数据库**
```bash
# 使用 Docker
docker-compose up -d db redis

# 或手动启动
# PostgreSQL 和 Redis 服务
```

5. **运行数据库迁移**
```bash
cd htyuc_models
diesel setup
diesel migration run
```

6. **启动服务**
```bash
# 启动用户中心服务
cargo run --bin htyuc

# 或使用提供的脚本
./htyuc/start.sh
```

### 验证安装

```bash
# 测试服务是否正常运行
curl http://localhost:3001/health

# 运行测试
cargo test
```

## 📁 项目结构

```
AuthCore/
├── htycommons/          # 通用工具库
│   ├── src/
│   │   ├── jwt.rs       # JWT 认证
│   │   ├── db.rs        # 数据库工具
│   │   ├── web.rs       # Web 工具
│   │   ├── wx.rs        # 微信集成
│   │   ├── redis_util.rs # Redis 工具
│   │   ├── cert.rs      # 证书管理
│   │   ├── upyun.rs     # 又拍云集成
│   │   └── ...
│   └── tests/           # 测试文件
├── htyuc/               # 用户中心服务
│   ├── src/
│   │   ├── main.rs      # 服务入口
│   │   ├── lib.rs       # 核心逻辑
│   │   └── ...
│   └── tests/           # 测试文件
├── htyuc_models/        # 数据模型
│   ├── migrations/      # 数据库迁移
│   └── src/
│       ├── models.rs    # 数据模型
│       └── schema.rs    # 数据库模式
├── htyuc_remote/        # 远程服务调用
├── certutil/            # 证书管理工具
├── upyun_tool/          # 又拍云上传工具
└── Cargo.toml           # 工作空间配置
```

## 📚 API 文档

### 用户认证 API

#### 用户注册
```http
POST /api/v1/uc/register
Content-Type: application/json

{
  "username": "user@example.com",
  "password": "secure_password",
  "nickname": "用户昵称"
}
```

#### 用户登录
```http
POST /api/v1/uc/login
Content-Type: application/json

{
  "username": "user@example.com",
  "password": "secure_password"
}
```

#### 获取用户信息
```http
GET /api/v1/uc/user
Authorization: Bearer <jwt_token>
```

#### 刷新令牌
```http
POST /api/v1/uc/refresh
Authorization: Bearer <jwt_token>
```

### 微信集成 API

#### 微信登录
```http
POST /api/v1/uc/wx/login
Content-Type: application/json

{
  "code": "wx_auth_code"
}
```

#### 获取微信用户信息
```http
GET /api/v1/uc/wx/userinfo
Authorization: Bearer <jwt_token>
```

### 文件上传 API

#### 上传文件
```http
POST /api/v1/uc/upload
Authorization: Bearer <jwt_token>
Content-Type: multipart/form-data

file: <file_data>
```

## ⚙️ 配置说明

### 环境变量

创建 `.env` 文件并配置以下环境变量：

```env
# 数据库配置
DATABASE_URL=postgres://username:password@localhost/authcore
UC_DB_URL=postgres://username:password@localhost/htyuc
WS_DB_URL=postgres://username:password@localhost/htyws

# Redis 配置
REDIS_HOST=localhost
REDIS_PORT=6379

# JWT 配置
JWT_KEY=your_jwt_secret_key_here
EXPIRATION_DAYS=30

# 服务配置
UC_PORT=3001
WS_PORT=3000
LOGGER_LEVEL=INFO
POOL_SIZE=20

# 微信配置
WEIXIN_APP_ID=your_weixin_app_id
WEIXIN_SECRET=your_weixin_secret

# 又拍云配置
UPYUN_OPERATOR=your_upyun_operator
UPYUN_PASSWORD=your_upyun_password

# 功能开关
SKIP_POST_LOGIN=false
SKIP_REGISTRATION=false
SKIP_WX_PUSH=false
```

### 数据库配置

1. **创建数据库**
```sql
CREATE DATABASE authcore;
CREATE DATABASE htyuc;
CREATE DATABASE htyws;
```

2. **运行迁移**
```bash
cd htyuc_models
diesel setup
diesel migration run
```

## 🛠️ 开发指南

### 开发环境设置

1. **安装 Rust 工具链**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

2. **安装 PostgreSQL 和 Redis**
```bash
# macOS
brew install postgresql redis

# Ubuntu
sudo apt-get install postgresql redis-server
```

3. **安装 Diesel CLI**
```bash
cargo install diesel_cli --no-default-features --features postgres
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test --package htycommons

# 运行测试并显示输出
print_debug=true cargo test -- --nocapture

# 运行测试并限制线程数
cargo test -- --test-threads=1
```

### 代码格式化

```bash
# 格式化代码
cargo fmt

# 检查代码风格
cargo clippy
```

### 构建项目

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 构建特定模块
cargo build --package htyuc
```

## 🧪 测试

### 单元测试

```bash
# 运行 htycommons 测试
cd htycommons
cargo test

# 运行 htyuc 测试
cd htyuc
cargo test
```

### 集成测试

```bash
# 运行完整的集成测试
cargo test --test integration_tests
```

### 性能测试

```bash
# 运行基准测试
cargo bench
```

## 📦 部署

### Docker 部署

1. **构建镜像**
```bash
docker build -t authcore .
```

2. **运行容器**
```bash
docker run -d \
  --name authcore \
  -p 3001:3001 \
  -e DATABASE_URL=postgres://user:pass@host/db \
  authcore
```

### 生产环境部署

1. **编译发布版本**
```bash
cargo build --release
```

2. **配置系统服务**
```bash
# 创建 systemd 服务文件
sudo cp scripts/authcore.service /etc/systemd/system/
sudo systemctl enable authcore
sudo systemctl start authcore
```

## 🤝 贡献指南

我们欢迎所有形式的贡献！请查看 [CONTRIBUTORS.md](CONTRIBUTORS.md) 了解项目贡献者历史。

### 贡献步骤

1. Fork 本项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

### 贡献类型

- 🐛 Bug 修复
- ✨ 新功能
- 📝 文档改进
- 🧪 测试用例
- ⚡ 性能优化
- 🔒 安全增强

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

感谢所有为 AuthCore 项目做出贡献的开发者。特别感谢：

- **liweinan (阿男)**: 项目架构师和主要开发者
- **Buddy119**: 核心协作开发者
- **xiaolitongxue666**: 功能开发者
- **Moicen**: 功能开发者
- **beyoung**: 贡献者
- **twainyoung**: 贡献者

详细贡献信息请查看 [CONTRIBUTORS.md](CONTRIBUTORS.md)。

## 📞 联系我们

- **项目主页**: [GitHub Repository](https://github.com/your-org/AuthCore)
- **问题反馈**: [Issues](https://github.com/your-org/AuthCore/issues)
- **讨论区**: [Discussions](https://github.com/your-org/AuthCore/discussions)

---

⭐ 如果这个项目对你有帮助，请给我们一个星标！
