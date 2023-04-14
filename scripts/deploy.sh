#!/usr/bin/env bash
set -eux

function deploy() {
  set -x
  local app_dir="/app"
  local app_name="optimus"
  local app_path="${app_dir}/${app_name}"
  local systemd_service_name="optimus-discord-bot"
  mkdir -p "${app_dir}"


  # Get rid of bloat
  for i in apt-daily.timer update-notifier-download.timer update-notifier-motd.timer; do
          systemctl disable $i
          systemctl stop $i
  done
  apt purge -yq snapd unattended-upgrades

  # Install systemd service units
  cat > "/etc/systemd/system/${systemd_service_name}.service" <<EOF
[Unit]
Description=Optimus discord Bot
After=network.target

[Service]
ExecStart=${app_path}
Restart=always

[Install]
WantedBy=multi-user.target

EOF

  systemctl daemon-reload
  systemctl enable "${systemd_service_name}"
  systemctl stop "${systemd_service_name}"

  mv "${DEPLOY_DIR}"/* "${app_dir}"
  mv "${app_dir}/ProdBotConfig.toml" "${app_dir}/BotConfig.toml"
  chmod +x "${app_dir}"/*

  systemctl start "${systemd_service_name}"
  rm -rf "${DEPLOY_DIR}"
}

cargo build --release

private_key=/tmp/.pkey
if test ! -e "$private_key"; then {
  base64 -d <<<"${PRIVATE_KEY_ENCODED}" > "$private_key"
  chmod 0600 "$private_key"
} fi
ssh_cmd=(
  ssh -i "${private_key}"
  -o UserKnownHostsFile=/dev/null
  -o StrictHostKeyChecking=no
  $SSH_LOGIN
)

tmp_deploy_dir=/tmp/deploy
rm -rf "$tmp_deploy_dir"
mkdir -p "$tmp_deploy_dir"
cp ./target/release/optimus ProdBotConfig.toml $(which meilisearch) "$tmp_deploy_dir"

tar -cf - "${tmp_deploy_dir}" | "${ssh_cmd[@]}" -- tar -C / -xf -
printf '%s\n' \
  "DEPLOY_DIR=${tmp_deploy_dir}" \
  "$(declare -f deploy)" \
  "deploy" | "${ssh_cmd[@]}" -- bash
