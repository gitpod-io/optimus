#!/usr/bin/env bash
set -eux

function deploy() {
  local app_dir="/app"
  local app_name="optimus"
  local app_path="${app_dir}/${app_name}"
  local systemd_service_name="optimus-discord-bot.service"
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
  systemctl start "${systemd_service_name}"
  rm -rf "${DEPLOY_DIR}"
}

cargo build --release

read -a ssh_cmd < <(gcloud compute ssh "discord-optimus-bot" --dry-run >&1)

tmp_deploy_dir=/tmp/deploy
mkdir -p "$tmp_deploy_dir"
cp ./target/release/optimus $(which meilisearch) "$tmp_deploy_dir"

tar -cf - "${tmp_deploy_dir}" | "${ssh_cmd[@]}" -- tar -C / -xf -
printf '%s\n' \
  "DEPLOY_DIR=${tmp_deploy_dir}" \
  "$(declare -f deploy)" \
  "deploy" | "${ssh_cmd[@]}" -- sudo bash
