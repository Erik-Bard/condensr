FROM node:22-slim AS builder
WORKDIR /build
RUN corepack enable
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY apps/desktop/package.json ./apps/desktop/package.json
RUN pnpm install --frozen-lockfile
COPY apps/desktop ./apps/desktop
RUN pnpm --dir apps/desktop build

FROM nginx:1.27-alpine
COPY apps/desktop/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /build/apps/desktop/dist /usr/share/nginx/html
EXPOSE 80
