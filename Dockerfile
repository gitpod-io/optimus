FROM getmeili/meilisearch:v1.1 as meilisearch_alpine

FROM alpine:3.16

ARG APP_DIR="/app"
ENV PATH="${APP_DIR}:${PATH}"
RUN mkdir -m 0755 -p "${APP_DIR}"

# Meilisearch
# ENV MEILI_DB_PATH="/app/data.ms"
# RUN url="https://github.com/meilisearch/meilisearch/releases/download/v1.1.0/meilisearch-linux-amd64" && \
# 	path="/app/meilisearch" \
# 	&& curl -L "${url}" -o "${path}" \
# 	&& chmod +x "${path}"
RUN apk add --no-cache libgcc
COPY --from=meilisearch_alpine --chown=root:root /bin/meilisearch ${APP_DIR}/meilisearch

# Bot
COPY target/x86_64-unknown-linux-musl/release/optimus ${APP_DIR}/optimus

# Automatically start meilisearch, and read a BotConfig.toml from ${APP_DIR} (if exists)
ENTRYPOINT ["optimus"]
