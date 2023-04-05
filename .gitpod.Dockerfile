FROM gitpod/workspace-rust:2023-03-24-22-45-37

# Install meilisearch
RUN target=/usr/bin/meilisearch \
  && sudo bash -c "curl -o ${target} -L https://github.com/meilisearch/meilisearch/releases/download/v1.1.0/meilisearch-linux-amd64 && chmod +x ${target}"
