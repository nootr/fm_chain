FROM rust:1.87-slim-bullseye AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir -p src && echo "fn main() {println!(\"<3\");}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

COPY . .

RUN cargo build --release --locked --target-dir /tmp/target

###
FROM debian:bookworm-slim

WORKDIR /app

ENV DATABASE_URL=sqlite:fm_chain.db
ENV HOST=0.0.0.0

COPY --from=builder /tmp/target/release/fm_chain .
COPY static ./static
COPY templates ./templates
COPY migrations ./migrations

RUN touch fm_chain.db

CMD ["./fm_chain"]
