FROM gitpod/workspace-rust:2023-03-24-22-45-37

# meilisearch
ENV MEILISEARCH_API_KEY="superCoolApiKey1234"

USER root

# Install meilisearch
RUN target=/usr/bin/meilisearch \
  && curl -o "${target}" \
  -L https://github.com/meilisearch/meilisearch/releases/download/v1.1.0/meilisearch-linux-amd64 \
  && chmod +x "${target}"

# Install gcloud CLI
RUN echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" \
  >> /etc/apt/sources.list.d/google-cloud-sdk.list \
  && curl https://packages.cloud.google.com/apt/doc/apt-key.gpg \
  | apt-key --keyring /usr/share/keyrings/cloud.google.gpg add - \
  && install-packages google-cloud-cli
