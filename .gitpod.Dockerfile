FROM gitpod/workspace-rust:2023-03-24-22-45-37

USER root

# Install meilisearch
RUN target=/usr/bin/meilisearch \
  && curl -o "${target}" \
  -L https://github.com/meilisearch/meilisearch/releases/download/v1.1.0/meilisearch-linux-amd64 \
  && chmod +x "${target}"

RUN rustup default nightly \
    && rustup component add clippy --toolchain nightly #-2023-07-13 \
    && rustup component add rustfmt --toolchain nightly #-2023-07-13

USER gitpod
