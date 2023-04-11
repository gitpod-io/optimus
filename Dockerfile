FROM ubuntu:22.04

ARG APP_DIR="/app"
ENV PATH="${APP_DIR}:${PATH}"
RUN mkdir -m 0755 -p "${APP_DIR}"

# Install curl
RUN apt-get update \
    && apt-get install --no-install-recommends -yq curl ca-certificates \
    && apt-get clean -y \
    && rm -rf /var/cache/debconf/* /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Bot
COPY target/release/optimus /app/optimus

# Meilisearch
# ENV MEILI_DB_PATH="/app/data.ms"
RUN url="https://github.com/meilisearch/meilisearch/releases/download/v1.1.0/meilisearch-linux-amd64" && \
    path="/app/meilisearch" \
    && curl -L "${url}" -o "${path}" \
    && chmod +x "${path}"

# Automatically start meilisearch, and reads a BotConfig.toml from /app dir (in exists)
ENTRYPOINT ["optimus"]
