FROM gitpod/workspace-rust:latest

USER root

# Install meilisearch
RUN target=/usr/bin/meilisearch \
  && curl -o "${target}" \
  -L https://github.com/meilisearch/meilisearch/releases/download/v1.1.0/meilisearch-linux-amd64 \
  && chmod +x "${target}"

USER gitpod

RUN rustup default nightly \
    && rustup component add clippy --toolchain nightly #-2023-07-13 \
    && rustup component add rustfmt --toolchain nightly #-2023-07-13
