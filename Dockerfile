# Passage Watch — production image.
# One Rust service serves the API and the built frontend from ./dist.

# ---------- frontend ----------
FROM node:22-bookworm-slim AS frontend
WORKDIR /app
RUN corepack enable && corepack prepare pnpm@10 --activate
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile
COPY tsconfig.json vite.config.ts ./
COPY index.html ./
COPY public ./public
COPY src ./src
RUN pnpm build

# ---------- backend ----------
FROM rust:1.91-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# Cache dependency compilation before the real sources arrive.
RUN mkdir -p src \
    && echo 'pub fn placeholder() {}' > src/lib.rs \
    && echo 'fn main() {}' > src/main.rs \
    && cargo build --release
COPY src ./src
RUN cargo build --release

# ---------- runtime ----------
FROM debian:bookworm-slim AS runtime
RUN apt-get update -qq \
    && apt-get install -y -qq --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /app/target/release/passage-watch /usr/local/bin/passage-watch
COPY --from=frontend /app/dist ./dist
COPY config/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh
ENV PORT=3000
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
