# Build stage: compile Rust backend
FROM rust:1.82-bookworm AS backend-builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

# Build stage: compile React frontend
FROM node:20-bookworm AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm install
COPY frontend/ ./
RUN npm run build

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libsqlite3-0 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend-builder /app/target/release/shmtu-service-monitor /app/shmtu-service-monitor
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist
EXPOSE 3100
CMD ["/app/shmtu-service-monitor"]
