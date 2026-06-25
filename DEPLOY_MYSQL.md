# ACMind MySQL 版本部署指南

## 🚀 服务器部署步骤

### 1. 准备服务器环境

确保服务器已安装：
- Docker
- Docker Compose

```bash
# 检查 Docker
docker --version
docker compose version
```

### 2. 克隆代码并切换到 MySQL 分支

```bash
git clone <repository-url>
cd acmind
git checkout mysql
```

### 3. 配置环境变量

编辑 `.env` 文件，确认数据库连接：

```bash
# Docker 环境使用容器名
DATABASE_URL=mysql://acmind:acmind@mysql:3306/acmind

# 生产环境建议修改：
JWT_SECRET=<生成一个强密码>
ALLOW_REGISTER=false  # 关闭公开注册

# 配置 AI（可选）
LLM_PROVIDER=openai
LLM_API_KEY=<your-api-key>
LLM_BASE_URL=https://api.openai.com/v1
LLM_MODEL=gpt-4o-mini
```

### 4. 启动所有服务

```bash
# 构建并启动
docker compose up -d --build

# 查看日志
docker compose logs -f

# 查看服务状态
docker compose ps
```

### 5. 访问应用

- **前端**: http://your-server-ip:5173
- **后端 API**: http://your-server-ip:8080
- **MySQL**: localhost:3306 (仅容器内部)

### 6. 停止服务

```bash
# 停止所有容器
docker compose down

# 停止并删除数据
docker compose down -v
```

## 🔧 生产环境优化

### 使用 Nginx 反向代理

```nginx
server {
    listen 80;
    server_name your-domain.com;

    # 前端
    location / {
        proxy_pass http://localhost:5173;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # API
    location /api/ {
        proxy_pass http://localhost:8080/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### 使用外部 MySQL 数据库

如果你有外部 MySQL 服务器：

1. 注释掉 `docker-compose.yml` 中的 `mysql` 服务
2. 修改 `.env`:
```bash
DATABASE_URL=mysql://user:password@external-host:3306/acmind
```

## 📊 数据库管理

### 备份数据

```bash
docker exec acmind-mysql-1 mysqldump -u acmind -pacmind acmind > backup.sql
```

### 恢复数据

```bash
docker exec -i acmind-mysql-1 mysql -u acmind -pacmind acmind < backup.sql
```

### 直接连接数据库

```bash
docker exec -it acmind-mysql-1 mysql -u acmind -pacmind acmind
```

## 🐛 故障排查

### 查看后端日志
```bash
docker compose logs api -f
```

### 查看数据库日志
```bash
docker compose logs mysql -f
```

### 重建容器
```bash
docker compose down
docker compose up -d --build --force-recreate
```

### 清理并重新开始
```bash
docker compose down -v  # 删除所有数据
docker compose up -d --build
```

## 📝 注意事项

1. ✅ 使用 MySQL 9.7（最新稳定版）
2. ✅ 数据持久化在 Docker volume `mysql_data` 中
3. ✅ 生产环境请修改默认密码
4. ✅ 建议定期备份数据库
5. ✅ 使用 HTTPS（配置 SSL 证书）
6. ✅ 配置防火墙，只开放必要端口

## 🔄 本地开发

如果要在本地开发（不用 Docker）：

1. 安装 MySQL: `sudo pacman -S mariadb` (Arch Linux)
2. 复制 `.env.local.example` 为 `.env.local`
3. 修改 `DATABASE_URL=mysql://acmind:acmind@localhost:3306/acmind`
4. 启动后端: `cargo run -p acmind-api`
5. 启动前端: `pnpm run dev`
