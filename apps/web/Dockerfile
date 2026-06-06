# Multi-stage build for the Vite + React web app.
FROM node:22-alpine AS builder
WORKDIR /app

# Copy manifests first for better layer caching.
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml turbo.json ./
COPY apps/web apps/web
COPY packages/shared packages/shared

RUN corepack enable && pnpm install --frozen-lockfile
RUN pnpm --filter @acmind/web build

# Serve the built static files with nginx.
FROM nginx:alpine
COPY --from=builder /app/apps/web/dist /usr/share/nginx/html
COPY apps/web/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
