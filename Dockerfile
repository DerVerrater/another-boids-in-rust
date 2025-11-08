FROM rust:1.89 AS builder

RUN apt-get update
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked wasm-bindgen-cli

COPY . .

RUN make -j

FROM busybox:musl
RUN mkdir -p /var/www
COPY --from=builder ./out/ /var/www
WORKDIR /var/www

# TODO: Make httpd accept interrupt signals so the container exits properly.
# (<ctrl>+c and `docker stop ...` don't work because Busybox doesn't answer)
CMD ["httpd", "-f", "-p", "8080"]
