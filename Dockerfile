# syntax=docker/dockerfile:1
#
# Open-FDD field-bus sidecar image.
#
# Mirrors the known-good local runtime: Python 3.12 with rusty-bacnet and
# rusty-haystack installed from PyPI (both ship prebuilt wheels). No Rust
# toolchain or source build required, so the image builds fast and reliably.
#
# rusty-modbus currently only ships a Python 3.14 wheel, so /modbus/* returns a
# clear "not installed" error on this image until that wheel lands on 3.12.
FROM python:3.12-slim AS runtime

ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONUNBUFFERED=1
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    iproute2 curl \
  && rm -rf /var/lib/apt/lists/*

# Pin the protocol wheels to the versions validated on the bench.
RUN pip install --no-cache-dir \
    "rusty-bacnet==0.10.1" \
    "rusty-haystack==0.7.2"

COPY pyproject.toml README.md ./
COPY app ./app
COPY config ./config
COPY scripts ./scripts

RUN pip install --no-cache-dir -e .

COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh scripts/preflight_free_47808.sh

EXPOSE 8080/tcp
EXPOSE 47808/udp

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
  CMD curl -fsS http://127.0.0.1:8080/health || exit 1

ENTRYPOINT ["/docker-entrypoint.sh"]
CMD ["python", "-m", "app.main"]
