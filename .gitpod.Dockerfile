FROM gitpod/workspace-rust:2023-10-13-07-50-14

USER root

# Install meilisearch
RUN target=/usr/bin/meilisearch \
  && curl -o "${target}" \
  -L https://github.com/meilisearch/meilisearch/releases/download/v1.1.0/meilisearch-linux-amd64 \
  && chmod +x "${target}"

USER gitpod

RUN rustup default nightly-2023-10-12 \
    && rustup component add clippy --toolchain nightly-2023-10-12 \
    && rustup component add rustfmt --toolchain nightly-2023-10-12
