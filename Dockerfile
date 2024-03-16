FROM rust:1.67 as builder
WORKDIR /app
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build --release


FROM debian:bullseye-slim as turunmap
RUN apt-get update
RUN apt-get install -y ca-certificates 
RUN rm -rf /var/lib/apt/lists/*
WORKDIR /app
EXPOSE 10204
COPY --from=builder /app/target/release/grepolis_diff_server /usr/bin/gregswatch
CMD ["/usr/bin/gregswatch"]
