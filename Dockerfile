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
COPY --from=meilisearch_alpine --chown=root:root /bin/meilisearch /app/meilisearch

# Bot
COPY target/x86_64-unknown-linux-musl/release/optimus /app/optimus
# EXPOSE 7700

# Automatically start meilisearch, and reads a BotConfig.toml from /app dir (in exists)
ENTRYPOINT ["optimus"]
