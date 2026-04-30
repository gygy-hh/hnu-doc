# HnuDoc

HNU 文档管理系统 / 后端服务

## 项目介绍

这是一个基于 Rust + Salvo 的 Web 服务项目，用于处理文档相关功能，包括 PDF 处理、JWT 鉴权、CAS 验证等。

### 主要特性
- Web 框架：Salvo 0.80
- 数据库：MySQL + SQLx
- 缓存：Redis
- 认证：JWT + AES 加密
- PDF 处理：lopdf
- 工作量证明 (PoW) 防刷
- 集成 spider_2024 的 CAS 密码验证

## 快速开始

### 1. 环境要求
- Rust 2024 edition
- MySQL
- Redis
- OpenSSL (非 Windows)

### 2. 配置
复制配置文件：
```bash
cp config/config.toml config/config.local.toml
```
然后修改 `config/config.local.toml` 中的数据库、Redis 和 JWT secret 等配置。

### 3. 运行
```bash
cargo run
```

服务默认运行在 `http://0.0.0.0:8080`

## 项目结构
```
.
├── Cargo.toml          # 工作空间配置
├── hnudoc/             # 主 crate
│   ├── Cargo.toml
│   └── src/
├── config/             # 配置文件
├── migrations/         # 数据库迁移
├── HnuDoc.md           # API 文档
└── README.md
```

## 部署

详见 `HnuDoc.md` 中的 API 文档。

## 贡献

欢迎提交 PR！

## License

MIT
