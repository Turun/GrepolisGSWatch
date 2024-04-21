FROM rust:1.67 as dependencies
WORKDIR /app
COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir -p src
RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release

FROM rust:1.67 as application
WORKDIR /app
COPY /Cargo.toml .
COPY /Cargo.lock .
COPY --from=dependencies /app/target/ /app/target
COPY --from=dependencies /usr/local/cargo /usr/local/cargo
COPY src/ src/
RUN cargo build --release

FROM debian:bullseye-slim as gregswatch
RUN apt-get update
RUN apt-get install -y ca-certificates 
RUN rm -rf /var/lib/apt/lists/*
EXPOSE 10204
COPY --from=application /app/target/release/gregswatch /gregswatch
CMD ["/gregswatch"]
