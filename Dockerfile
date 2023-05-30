FROM --platform=$BUILDPLATFORM rust:1.69 as builder

ENV CARGO_TERM_COLOR=always \
  CARGO_NET_GIT_FETCH_WITH_CLI=true

WORKDIR /app

ARG BUILDPLATFORM
RUN echo "BUILDPLATFORM: $BUILDPLATFORM"

ARG TARGETPLATFORM
RUN echo "TARGETPLATFORM: $TARGETPLATFORM"

RUN mkdir -p .cargo ; \
  echo '[build]' > .cargo/config.toml ; \
  case "$TARGETPLATFORM" in \
  "linux/amd64") \
  echo 'target = "x86_64-unknown-linux-gnu"' >> .cargo/config.toml \
  ;; \
  "linux/arm64") \
  echo 'target = "aarch64-unknown-linux-gnu"' >> .cargo/config.toml \
  ;; \
  esac ; \
  cat .cargo/config.toml

RUN apt update && apt install --assume-yes --no-install-recommends  g++-aarch64-linux-gnu libc6-dev-arm64-cross libudev-dev

RUN rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
RUN rustup toolchain install stable-aarch64-unknown-linux-gnu

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
  CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
  CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
  PKG_CONFIG_PATH="/usr/lib/aarch64-linux-gnu/pkgconfig/:${PKG_CONFIG_PATH}"

COPY . .

RUN cargo install --path . --root /usr/local

FROM debian:bullseye-slim AS runtime
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
WORKDIR /app
COPY --from=builder /usr/local/bin/jarvis-sma-planner .
ENTRYPOINT ["./jarvis-sma-planner"]