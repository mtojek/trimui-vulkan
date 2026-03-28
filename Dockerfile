FROM ghcr.io/loveretro/tg5040-toolchain:modernize

USER root

RUN apt-get update
RUN apt-get install -y --no-install-recommends ca-certificates curl tar
RUN apt-get install -y --no-install-recommends libglfw3-dev pkg-config
RUN rm -rf /var/lib/apt/lists/*
RUN curl -sSL https://go.dev/VERSION?m=text | head -n1 > /tmp/GO_VERSION
RUN uname -m > /tmp/ARCH
RUN set -e; \
    ARCH="$(cat /tmp/ARCH)"; \
    case "$ARCH" in \
      x86_64) echo "amd64" > /tmp/GO_ARCH ;; \
      aarch64|arm64) echo "arm64" > /tmp/GO_ARCH ;; \
      *) echo "Unsupported arch: $ARCH" >&2; exit 1 ;; \
    esac
RUN rm -rf /usr/local/go
RUN set -e; \
    GO_VERSION="$(cat /tmp/GO_VERSION)"; \
    GO_ARCH="$(cat /tmp/GO_ARCH)"; \
    curl -sSL "https://dl.google.com/go/${GO_VERSION}.linux-${GO_ARCH}.tar.gz" | tar -C /usr/local -xz

ENV PATH="/usr/local/go/bin:${PATH}"

RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable

ENV PATH="/root/.cargo/bin:${PATH}"

RUN mkdir -p /workspace
WORKDIR /workspace

CMD ["/bin/bash"]
