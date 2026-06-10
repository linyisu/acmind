# 部署指南

本地开发，推到服务器运行。

## 一次性服务器配置

```bash
# 1. 准备服务器（假设有 Docker + git）
ssh user@your-server

# 2. 克隆代码
git clone https://github.com/你的用户名/acmind.git ~/acmind
cd ~/acmind

# 3. 配置环境变量
cp .env.example .env
nano .env   # 至少修改 JWT_SECRET、LLM_API_KEY 等

# 4. 首次启动（自动建表、跑迁移）
docker compose up -d --build
```

服务器上的 `.env` 只需要配置一次，后续更新不会被覆盖。

## 日常部署流程

本地开发完，推代码：

```bash
# 本地
git add -A
git commit -m "feat: xxx"
git push origin main
```

然后去服务器拉取：

```bash
ssh user@your-server
cd ~/acmind

# 拉最新代码
git pull origin main

# 重新构建并重启 API（web 容器代码静态打包在镜像里）
docker compose up -d --build api
```

> **注意：不要用 `docker compose down -v`** — `-v` 会删除数据卷，所有用户数据都没了。
> 日常更新只用 `docker compose up -d --build <service>`。

## 首次 / 完整初始化

```bash
# 如果是全新服务器（数据库也要从零建）
docker compose down
docker compose up -d --build
```

## 常用命令

```bash
# 查看日志
docker compose logs -f api
docker compose logs -f web

# 进入 API 容器调试
docker compose exec api bash

# 直接连数据库
docker compose exec postgres psql -U acmind acmind

# 备份数据库
docker compose exec postgres pg_dump -U acmind acmind > backup-$(date +%F).sql

# 恢复数据库
cat backup-2026-06-10.sql | docker compose exec -T postgres psql -U acmind acmind
```

## 注意事项

- **数据卷 `postgres_data`** — 包含所有用户数据，除非显式删除否则一直保留
- **环境变量** — 服务器上的 `.env` 是「真实配置」，本地 `.env` 是「开发配置」，两套互不影响
- **端口冲突** — 默认 web 走 5173、API 走 8080、Postgres 走 5432，服务器上如果被占可以改 `.env`
- **HTTPS** — 当前 docker-compose 没有反向代理，生产环境建议前面加 nginx/caddy 套一层
