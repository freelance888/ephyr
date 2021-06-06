#!/usr/bin/env bash

set -e

EPHYR_CLI_ARGS=${EPHYR_CLI_ARGS:-''}
EPHYR_VER=${EPHYR_VER:-'0.3.0'}
if [ "$EPHYR_VER" == "latest" ]; then
  EPHYR_VER=''
else
  EPHYR_VER="-$EPHYR_VER"
fi


# Install Podman for running containers.
echo "deb https://download.opensuse.org/repositories/devel:/kubic:/libcontainers:/stable/xUbuntu_20.04/ /" \
  | tee /etc/apt/sources.list.d/devel:kubic:libcontainers:stable.list
curl -L https://download.opensuse.org/repositories/devel:/kubic:/libcontainers:/stable/xUbuntu_20.04/Release.key \
  | apt-key add -
apt-get -y update
apt-get -y install podman

WITH_FIREWALLD=${WITH_FIREWALLD:-0}
if [ "$WITH_FIREWALLD" == "1" ]; then
  # Install and setup firewalld, if required.
  apt-get -y install firewalld
  systemctl unmask firewalld.service
  systemctl enable firewalld.service
  systemctl start firewalld.service
  firewall-cmd --zone=public --permanent \
               --add-port=80/tcp --add-port=1935/tcp --add-port=8000/tcp
  firewall-cmd --reload
fi


# Install Ephyr re-streamer runner wrapper which detect directory for DVR.
cat <<'EOF' > /usr/local/bin/detect-ephyr-restreamer-volume.sh
#!/usr/bin/env bash

set -e

export EPHYR_WWW_DIR="/var/run/ephyr-restreamer/www"
do_volume="$(set +e; find /mnt/volume_* -type d | head -1 | tr -d '\n')"
if [ -d "$do_volume" ]; then
  export EPHYR_WWW_DIR="$do_volume/www"
fi
hcloud_volume="$(set +e; find /mnt/HC_Volume_* -type d | head -1 | tr -d '\n')"
if [ -d "$hcloud_volume" ]; then
  export EPHYR_WWW_DIR="$hcloud_volume/www"
fi

mkdir -p "$EPHYR_WWW_DIR/"

exec "$@"
EOF
chmod +x /usr/local/bin/detect-ephyr-restreamer-volume.sh


# Install Ephyr-restreamer runner (need pass $EPHYR_WWW_DIR).
cat <<'EOF' > /usr/local/bin/run-ephyr-restreamer.sh
#!/usr/bin/env bash

/usr/bin/podman run \\
  --network=host \\
  -v /var/lib/\${EPHYR_CONTAINER_NAME}/srs.conf:/usr/local/srs/conf/srs.conf \\
  -v /var/lib/\${EPHYR_CONTAINER_NAME}/state.json:/state.json \\
  -v \${EPHYR_WWW_DIR}/:/var/www/srs/ \\
  --name=\${EPHYR_CONTAINER_NAME} \\
  \${EPHYR_IMAGE_NAME}:\${EPHYR_IMAGE_TAG} ${EPHYR_CLI_ARGS}
EOF
chmod +x /usr/local/bin/run-ephyr-restreamer.sh


# Install Ephyr re-streamer SystemD Service.
cat <<EOF > /etc/systemd/system/ephyr-restreamer.service
[Unit]
Description=Ephyr service for re-streaming RTMP streams
After=local-fs.target podman.service
Requires=local-fs.target


[Service]
Environment=EPHYR_CONTAINER_NAME=ephyr-restreamer
Environment=EPHYR_IMAGE_NAME=docker.io/allatra/ephyr
Environment=EPHYR_IMAGE_TAG=restreamer${EPHYR_VER}

ExecStartPre=/usr/bin/mkdir -p /var/lib/\${EPHYR_CONTAINER_NAME}/
ExecStartPre=touch /var/lib/\${EPHYR_CONTAINER_NAME}/srs.conf
ExecStartPre=touch /var/lib/\${EPHYR_CONTAINER_NAME}/state.json

ExecStartPre=-/usr/bin/podman pull \${EPHYR_IMAGE_NAME}:\${EPHYR_IMAGE_TAG}
ExecStartPre=-/usr/bin/podman stop \${EPHYR_CONTAINER_NAME}
ExecStartPre=-/usr/bin/podman rm --volumes \${EPHYR_CONTAINER_NAME}
ExecStart=/usr/local/bin/detect-ephyr-restreamer-volume.sh \\
  /usr/local/bin/run-ephyr-restreamer.sh
ExecStop=-/usr/bin/podman stop \${EPHYR_CONTAINER_NAME}
ExecStop=-/usr/bin/podman rm --volumes \${EPHYR_CONTAINER_NAME}

Restart=always


[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl unmask ephyr-restreamer.service
systemctl enable ephyr-restreamer.service
systemctl restart ephyr-restreamer.service
