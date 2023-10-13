FROM gitpod/workspace-base:2023-10-13-07-50-14

USER root

# Install meilisearch
RUN target=/usr/bin/meilisearch \
  && curl -o "${target}" \
  -L https://github.com/meilisearch/meilisearch/releases/download/v1.1.0/meilisearch-linux-amd64 \
  && chmod +x "${target}"

USER gitpod

ENV RUST_VERSION="nightly-2023-10-12"
ENV PATH=$HOME/.cargo/bin:$PATH

RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path --default-toolchain ${RUST_VERSION} \
  -c rls rust-analysis rust-src rustfmt clippy \
  && for cmp in rustup cargo; do rustup completions bash "$cmp" > "$HOME/.local/share/bash-completion/completions/$cmp"; done \
  && printf '%s\n'    'export CARGO_HOME=/workspace/.cargo' \
  'mkdir -m 0755 -p "$CARGO_HOME/bin" 2>/dev/null' \
  'export PATH=$CARGO_HOME/bin:$PATH' \
  'test ! -e "$CARGO_HOME/bin/rustup" && mv "$(command -v rustup)" "$CARGO_HOME/bin"' > $HOME/.bashrc.d/80-rust \
  && rm -rf "$HOME/.cargo/registry" # This registry cache is now useless as we change the CARGO_HOME path to `/workspace`

RUN rustup default ${RUST_VERSION} # not needed but anyway \
    && rustup component add clippy \
    && rustup component add rustfmt
