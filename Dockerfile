# syntax=docker/dockerfile:1

# ---- Builder: compile PyO3 wheels (Python 3.14) ----
FROM rust:1.95-bookworm AS rust-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    python3.14 python3.14-dev python3.14-venv python3-pip \
    curl build-essential pkg-config libssl-dev \
  && rm -rf /var/lib/apt/lists/*

RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.local/bin:${PATH}"

RUN pip3 install --break-system-packages maturin

WORKDIR /build

# rusty-bacnet wheel
COPY rusty-bacnet /build/rusty-bacnet
RUN cd /build/rusty-bacnet/crates/rusty-bacnet && \
    maturin build --release -o /wheels

# rusty-haystack wheel
COPY rusty-haystack /build/rusty-haystack
RUN cd /build/rusty-haystack/rusty-haystack && \
    maturin build --release -o /wheels

# rusty-modbus wheel (clone dev branch)
RUN git clone --depth 1 --branch dev https://github.com/jscott3201/rusty-modbus.git /build/rusty-modbus
RUN cd /build/rusty-modbus/crates/rusty-modbus-python && \
    maturin build --release -o /wheels

# ---- Runtime ----
FROM python:3.14-slim AS runtime

ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONUNBUFFERED=1
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    iproute2 curl \
  && rm -rf /var/lib/apt/lists/*

COPY --from=rust-builder /wheels /wheels
COPY diy-bacnet-server/pyproject.toml diy-bacnet-server/README.md ./
COPY diy-bacnet-server/app ./app
COPY diy-bacnet-server/config ./config
COPY diy-bacnet-server/scripts ./scripts

RUN pip install --no-cache-dir /wheels/*.whl && \
    pip install --no-cache-dir -e .

COPY diy-bacnet-server/docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh scripts/preflight_free_47808.sh

EXPOSE 8080/tcp
EXPOSE 47808/udp

ENTRYPOINT ["/docker-entrypoint.sh"]
CMD ["python", "-m", "app.main"]
