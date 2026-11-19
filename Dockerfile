# Passage Watch — production image. Builds frontend, then serves backend+static.
FROM rust:1.91-bookworm AS build

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo 'pub fn placeholder() {}' > src/lib.rs && echo 'fn main() {}' > src/main.rs && cargo build --release 2>&1 | tail -2

# Replace placeholders with the real sources and rebuild.
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS frontend
WORKDIR /app
COPY package.json pnpm-lock.yaml ./
COPY pnpm-workspace.yaml ./
RUN apt-get update -qq && apt-get install -y -qq nodejs npm && npm install -g pnpm@10 && pnpm --version
COPY src ./src
COPY index.html package.json vite.config.ts tsconfig.json ./
COPY public ./public
RUN pnpm install --frozen-lockfile && pnpm build

FROM debian:bookworm-slim AS runtime
RUN apt-get update -qq && apt-get install -y -qq ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /app/target/release/passage-watch /usr/local/bin/passage-watch
COPY --from=frontend /app/dist ./dist
COPY config/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
