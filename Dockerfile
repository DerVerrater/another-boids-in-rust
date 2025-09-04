FROM rust:1.89 as builder

RUN apt-get update
RUN apt-get install -y --no-install-recommends libasound2-dev libudev-dev
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked wasm-bindgen-cli

# Copy only the package manifest and source code. Otherwise changes to anything
# will cause Docker to re-run `cargo build` even when the source hasn't changed.
COPY src/ ./src
COPY Cargo.toml ./Cargo.toml
COPY Cargo.lock

RUN cargo build --locked --target wasm32-unknown-unknown --profile=wasm-release
RUN wasm-bindgen --no-typescript --target web --out-dir ./out/ --out-name "boids" target/wasm32-unknown-unknown/wasm-release/another-boids-in-rust.wasm
COPY www/index.html out/index.html

FROM busybox:musl
RUN mkdir -p /var/www
COPY --from=builder ./out/ /var/www
WORKDIR /var/www

# TODO: Make httpd accept interrupt signals so the container exits properly.
# (<ctrl>+c and `docker stop ...` don't work because Busybox doesn't answer)
CMD ["httpd", "-f", "-p", "8080"]
