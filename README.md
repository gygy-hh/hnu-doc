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

### 2. 本地依赖（推荐 Docker）

在项目根目录启动 MySQL 与 Redis：

```bash
docker compose up -d
```

**Windows：** 若尚未安装 Docker，请先安装 [Docker Desktop](https://docs.docker.com/desktop/setup/install/windows-install/)（需启用 WSL 2 或 Hyper-V，安装后**重启电脑或注销**，再打开终端）。

创建数据库表：

```powershell
.\scripts\init-db.ps1
```

若提示脚本无法运行，可使用 **CMD** 运行（不依赖 PowerShell 策略）：

```cmd
scripts\init-db.cmd
```

若尚未创建本地配置，可复制示例：

```powershell
Copy-Item config/config.local.example.toml config/config.local.toml
```

程序会**优先**读取 `config/config.local.toml`（该文件已加入 `.gitignore`），再回退到 `config/config.toml`。示例里的账号密码与 `docker-compose.yml` 中的 MySQL 一致。

本地测试可在 `config.local.toml` 中启用 **`[dev].mock_login`**（见 `config.local.example.toml`）：使用固定学号/密码跳过 CAS，**切勿在生产环境开启**。

### 3. 配置（自建 MySQL / Redis）

复制配置文件：

```bash
cp config/config.toml config/config.local.toml
```

然后修改 `config/config.local.toml` 中的数据库、Redis 和 JWT secret 等配置。

### 4. 运行
```bash
cargo run -p hnudoc
```

服务默认运行在 `http://0.0.0.0:8080`（本机访问请用 **`http://127.0.0.1:8080`**）。

验证码解析服务（大物实验等）仅在调试相关接口时需要；本地可将 `captcha_url` 指向自建服务，否则不必启动。

### 5. 前端（Next.js）联调

前端仓库已克隆在 **`hnu-doc-frontend/`**（来源：[qnxg/hnu-doc-frontend](https://github.com/qnxg/hnu-doc-frontend)）。

1. 先按上文启动 MySQL、Redis 并运行后端（`cargo run -p hnudoc`）。
2. 前端目录安装依赖并启动：

```powershell
cd hnu-doc-frontend
pnpm install
pnpm dev
```

3. 接口地址由 **`NEXT_PUBLIC_BACKEND_URL`** 指定；仓库内已提供 **`hnu-doc-frontend/.env.example`**，本地可复制为 `.env.local`（默认 `http://127.0.0.1:8080`）。

浏览器打开 **http://localhost:3000** 即可联调登录与接口。

## 项目结构
```
.
├── Cargo.toml          # 工作空间配置
├── docker-compose.yml  # 本地 MySQL / Redis（可选）
├── hnudoc/             # 主 crate
│   ├── Cargo.toml
│   └── src/
├── config/             # 配置文件
├── scripts/            # 辅助脚本（如 init-db.ps1）
├── migrations/         # 数据库迁移
├── hnudoc.json         # Apifox 导出 OpenAPI（可选）
├── hnu-doc-frontend/   # Next.js 前端
├── HnuDoc.md           # API 文档
└── README.md
```

## 部署

详见 `HnuDoc.md` 中的 API 文档。

## 贡献

欢迎提交 PR！

## License

MIT
