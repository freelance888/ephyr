#!/usr/bin/env bash

set -e

EPHYR_CLI_ARGS=${EPHYR_CLI_ARGS:-''}
EPHYR_VER=${EPHYR_VER:-'0.6.0'}

# If want to use custom Docker registry
REGISTRY_USER=${REGISTRY_USER:-0}
REGISTRY_PASSWORD=${REGISTRY_PASSWORD:-0}
REGISTRY_URL=${REGISTRY_URL:-'docker.io'}

# If provider require firewalld instead of ufw (Oracle for example)
WITH_FIREWALLD=${WITH_FIREWALLD:-0}

# If provider require full update before install (Selectel for example)
WITH_INITIAL_UPGRADE=${WITH_INITIAL_UPGRADE:-0}

# If want to send traces to Jaeger
EPHYR_RESTREAMER_JAEGER_AGENT_IP=${EPHYR_RESTREAMER_JAEGER_AGENT_IP:-0}
EPHYR_RESTREAMER_JAEGER_AGENT_PORT=${EPHYR_RESTREAMER_JAEGER_AGENT_PORT:-0}
EPHYR_RESTREAMER_JAEGER_SERVICE_NAME=${EPHYR_RESTREAMER_JAEGER_SERVICE_NAME:-$(hostname)}

if [ "$EPHYR_VER" == "latest" ]; then
  EPHYR_VER=''
else
  EPHYR_VER="-$EPHYR_VER"
fi

if [ "$WITH_INITIAL_UPGRADE" == "1" ]; then
    apt-get -qy update
    DEBIAN_FRONTEND=noninteractive \
        apt-get -qy -o "Dpkg::Options::=--force-confdef" \
                    -o "Dpkg::Options::=--force-confold" upgrade
fi

# Install Docker for running containers.
apt-get -qy update
curl -sL https://get.docker.com | bash -s

# Login to custom Docker Registry if provided
if [[ "$REGISTRY_USER" != 0 && "$REGISTRY_PASSWORD" != 0 && "$REGISTRY_URL" != "docker.io" ]]; then
  docker login -u "$REGISTRY_USER" -p "$REGISTRY_PASSWORD" "$REGISTRY_URL"
fi


if [ "$WITH_FIREWALLD" == "1" ]; then
  # Install and setup firewalld, if required.
  apt-get -qy install firewalld
  systemctl unmask firewalld.service
  systemctl enable firewalld.service
  systemctl start firewalld.service
  firewall-cmd --zone=public --permanent \
               --add-port=80/tcp --add-port=1935/tcp --add-port=8000/tcp
  firewall-cmd --reload
else
  # Open default ports
  apt-get -yq install ufw
  ufw allow 80/tcp
  ufw allow 8000/tcp
  ufw allow 1935/tcp
fi


# Install Ephyr-restreamer runner
cat <<'EOF' > /usr/local/bin/run-ephyr-restreamer.sh
#!/usr/bin/env bash

set -e

# Detect directory for DVR.
ephyr_www_dir="/usr/local/share/ephyr-restreamer/www"
do_volume="$(set +e; find /mnt/volume_* -type d | head -1 | tr -d '\n')"
if [ -d "$do_volume" ]; then
  ephyr_www_dir="$do_volume/www"
fi
hcloud_volume="$(set +e; find /mnt/HC_Volume_* -type d | head -1 | tr -d '\n')"
if [ -d "$hcloud_volume" ]; then
  ephyr_www_dir="$hcloud_volume/www"
fi

echo "ephyr_www_dir=$ephyr_www_dir"
mkdir -p "$ephyr_www_dir/"

# Print all required Environment variables.
echo "EPHYR_IMAGE_TAG=$EPHYR_IMAGE_TAG"
echo "EPHYR_CLI_ARGS=$EPHYR_CLI_ARGS"
echo "EPHYR_CONTAINER_NAME=$EPHYR_CONTAINER_NAME"
echo "EPHYR_IMAGE_NAME=$EPHYR_IMAGE_NAME"
echo "EPHYR_RESTREAMER_STATE_PATH=$EPHYR_RESTREAMER_STATE_PATH"

# Print Jaeger related Environment variables.
echo "EPHYR_RESTREAMER_JAEGER_AGENT_IP=$EPHYR_RESTREAMER_JAEGER_AGENT_IP"
echo "EPHYR_RESTREAMER_JAEGER_AGENT_PORT=$EPHYR_RESTREAMER_JAEGER_AGENT_PORT"
echo "EPHYR_RESTREAMER_JAEGER_SERVICE_NAME=$EPHYR_RESTREAMER_JAEGER_SERVICE_NAME"

# Run Docker service
/usr/bin/docker run \
  --network=host \
  -v /var/lib/$EPHYR_CONTAINER_NAME/srs.conf:/usr/local/srs/conf/srs.conf \
  -v /var/lib/$EPHYR_CONTAINER_NAME/state.json:$EPHYR_RESTREAMER_STATE_PATH \
  -v $ephyr_www_dir/:/var/www/srs/ \
  -e EPHYR_RESTREAMER_JAEGER_AGENT_IP=$EPHYR_RESTREAMER_JAEGER_AGENT_IP \
  -e EPHYR_RESTREAMER_JAEGER_AGENT_PORT=$EPHYR_RESTREAMER_JAEGER_AGENT_PORT \
  -e EPHYR_RESTREAMER_JAEGER_SERVICE_NAME=$EPHYR_RESTREAMER_JAEGER_SERVICE_NAME \
  --name=$EPHYR_CONTAINER_NAME \
  $EPHYR_IMAGE_NAME:$EPHYR_IMAGE_TAG $EPHYR_CLI_ARGS
EOF
chmod +x /usr/local/bin/run-ephyr-restreamer.sh


# Install Ephyr re-streamer SystemD Service.
cat <<EOF > /etc/systemd/system/ephyr-restreamer.service
[Unit]
Description=Ephyr service for re-streaming RTMP streams
After=local-fs.target docker.service
Requires=local-fs.target


[Service]
Environment=EPHYR_CONTAINER_NAME=ephyr-restreamer
Environment=EPHYR_IMAGE_NAME=${REGISTRY_URL}/allatra/ephyr
Environment=EPHYR_IMAGE_TAG=restreamer${EPHYR_VER}
Environment=EPHYR_RESTREAMER_STATE_PATH=/tmp/workdir/state.json
# Jaeger configs
Environment=EPHYR_RESTREAMER_JAEGER_AGENT_IP=${EPHYR_RESTREAMER_JAEGER_AGENT_IP}
Environment=EPHYR_RESTREAMER_JAEGER_AGENT_PORT=${EPHYR_RESTREAMER_JAEGER_AGENT_PORT}
Environment=EPHYR_RESTREAMER_JAEGER_SERVICE_NAME=${EPHYR_RESTREAMER_JAEGER_SERVICE_NAME}

ExecStartPre=/usr/bin/mkdir -p /var/lib/\${EPHYR_CONTAINER_NAME}/
ExecStartPre=touch /var/lib/\${EPHYR_CONTAINER_NAME}/srs.conf
ExecStartPre=touch /var/lib/\${EPHYR_CONTAINER_NAME}/state.json

ExecStartPre=-/usr/bin/docker pull \${EPHYR_IMAGE_NAME}:\${EPHYR_IMAGE_TAG}
ExecStartPre=-/usr/bin/docker stop \${EPHYR_CONTAINER_NAME}
ExecStartPre=-/usr/bin/docker rm --volumes \${EPHYR_CONTAINER_NAME}
ExecStart=/usr/local/bin/run-ephyr-restreamer.sh
ExecStop=-/usr/bin/docker stop \${EPHYR_CONTAINER_NAME}
ExecStop=-/usr/bin/docker rm --volumes \${EPHYR_CONTAINER_NAME}

Restart=always


[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl unmask ephyr-restreamer.service
systemctl enable ephyr-restreamer.service
systemctl restart ephyr-restreamer.service
