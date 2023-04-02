#!/usr/bin/env bash

# This script automates the setup and configuration of Ephyr-restreamer on a Linux system.
# It installs Docker, sets up the firewall, and creates the required directories and configuration files.
# The script also installs the Ephyr-restreamer executor and a SystemD service to manage the Ephyr-restreamer Docker container.
#
# You can customize the script behavior by setting the following environment variables before running the script:
#
# 1. EPHYR_VER: Set the Ephyr-restreamer version. Default is '0.6.0'. Set to 'latest' for the latest version.
# 2. REGISTRY_URL: Set the Docker registry URL. Default is 'docker.io'.
# 3. REGISTRY_USER: Set the Docker registry username if using a custom registry.
# 4. REGISTRY_PASSWORD: Set the Docker registry password if using a custom registry.
# 5. EPHYR_CLI_ARGS: Set any additional CLI arguments for the Ephyr-restreamer Docker container.
# 6. WITH_INITIAL_UPGRADE: Set to '1' if the system requires a full update before installing (e.g., for Selectel). Default is '0'.
# 7. WITH_FIREWALLD: Set to '1' if the system requires firewalld instead of ufw (e.g., for Oracle). Default is '0'.
# 8. EPHYR_RESTREAMER_JAEGER_AGENT_IP: Set the IP address of the Jaeger agent if you want to send traces to Jaeger.
# 9. EPHYR_RESTREAMER_JAEGER_AGENT_PORT: Set the port of the Jaeger agent if you want to send traces to Jaeger.
# 10. EPHYR_RESTREAMER_JAEGER_SERVICE_NAME: Set the Jaeger service name for the Ephyr-restreamer traces. Default is the hostname of the machine.
# 11. CLEAR_STATE_ON_RESTART: Clear `state.json` each restart of Ephyr-restreamer. Default is '0'.
# 12. ALLOWED_IPS: Set allowed IP addresses to access server. Default is '*'.
#
# Example usage:
#   EPHYR_VER=latest WITH_INITIAL_UPGRADE=1 ./ubuntu-20-04-x64.sh


set -e

function get_ephyr_version() {
    local EPHYR_VER="${EPHYR_VER:-'0.6.0'}"
    if [ "$EPHYR_VER" == "latest" ]; then
        EPHYR_VER=''
    else
        EPHYR_VER="-$EPHYR_VER"
    fi
    echo "$EPHYR_VER"
}

function update_and_install_docker() {
    apt-get -qy update
    curl -sL https://get.docker.com | bash -s
}

function upgrade_if_required() {
    # If provider require full update before install (Selectel for example)
    local WITH_INITIAL_UPGRADE="${WITH_INITIAL_UPGRADE:-0}"
    if [ "$WITH_INITIAL_UPGRADE" == "1" ]; then
        DEBIAN_FRONTEND=noninteractive \
            apt-get -qy -o "Dpkg::Options::=--force-confdef" \
                        -o "Dpkg::Options::=--force-confold" upgrade
    fi
}

function login_to_registry_if_required() {
    # If want to use custom Docker registry
    local REGISTRY_URL="$1"

    local REGISTRY_USER="${REGISTRY_USER:-0}"
    local REGISTRY_PASSWORD="${REGISTRY_PASSWORD:-0}"

    if [[ "$REGISTRY_USER" != 0 && "$REGISTRY_PASSWORD" != 0 && "$REGISTRY_URL" != "docker.io" ]]; then
        docker login -u "$REGISTRY_USER" -p "$REGISTRY_PASSWORD" "$REGISTRY_URL"
    fi
}

function setup_firewall_rules() {
  # If ALLOWED_IPS is not set, use an empty array to allow all IPs
  local ALLOWED_IPS=(${ALLOWED_IPS:-"*"})

  # If provider require firewalld instead of ufw (Oracle for example)
  local WITH_FIREWALLD=${WITH_FIREWALLD:-0}
  local ALLOWED_PORTS=("80" "8000" "443" "1935" "3000")

  if [ "$WITH_FIREWALLD" == "1" ]; then
    # Install and setup firewalld, if required.
    apt-get -qy install firewalld
    systemctl unmask firewalld.service
    systemctl enable firewalld.service
    systemctl start firewalld.service

    firewall-cmd --zone=public --permanent --add-port="22/tcp"

    if [ "${#ALLOWED_IPS[@]}" == "*" ]; then
      for port in "${ALLOWED_PORTS[@]}"; do
        firewall-cmd --zone=public --permanent --add-port="${port}/tcp"
      done
    else
      for ip in "${ALLOWED_IPS[@]}"; do
        for port in "${ALLOWED_PORTS[@]}"; do
          firewall-cmd --permanent --zone=public --add-rich-rule="rule family='ipv4' source address='$ip' port port='$port' protocol='tcp' accept"
        done
      done
    fi
    firewall-cmd --reload
  else
    # Open default ports
    apt-get -yq install ufw
    ufw --force reset

    ufw allow 22
    ufw default deny incoming
    ufw default allow outgoing

    if [ "${ALLOWED_IPS}" == "*" ]; then
      for port in "${ALLOWED_PORTS[@]}"; do
        ufw allow "${port}/tcp"
      done
    else
      for ip in "${ALLOWED_IPS[@]}"; do
        for port in "${ALLOWED_PORTS[@]}"; do
          ufw allow from "$ip" to any port "$port"
        done
      done
    fi
    ufw --force enable
  fi
}


function setup_runtime_config {
  local ENV_FILE_PATH="$1"
  local STATE_PATH="$2"

  # If want to send traces to Jaeger
  local EPHYR_RESTREAMER_JAEGER_AGENT_IP=${EPHYR_RESTREAMER_JAEGER_AGENT_IP:-0}
  local EPHYR_RESTREAMER_JAEGER_AGENT_PORT=${EPHYR_RESTREAMER_JAEGER_AGENT_PORT:-0}
  local EPHYR_RESTREAMER_JAEGER_SERVICE_NAME=${EPHYR_RESTREAMER_JAEGER_SERVICE_NAME:-0}

  # Set environment for docker only if variables set.
  if [[ "$EPHYR_RESTREAMER_JAEGER_SERVICE_NAME" != "0" ]]; then
    echo "EPHYR_RESTREAMER_JAEGER_SERVICE_NAME=${EPHYR_RESTREAMER_JAEGER_SERVICE_NAME}" > "$ENV_FILE_PATH"
  else
    echo "EPHYR_RESTREAMER_JAEGER_SERVICE_NAME=$(hostname)" > "$ENV_FILE_PATH"
  fi
  if [[ "$EPHYR_RESTREAMER_JAEGER_AGENT_IP" != "0" && "$EPHYR_RESTREAMER_JAEGER_AGENT_PORT" != "0" ]]; then
    echo "EPHYR_RESTREAMER_JAEGER_AGENT_IP=${EPHYR_RESTREAMER_JAEGER_AGENT_IP}" >> "$ENV_FILE_PATH"
    echo "EPHYR_RESTREAMER_JAEGER_AGENT_PORT=${EPHYR_RESTREAMER_JAEGER_AGENT_PORT}" >> "$ENV_FILE_PATH"
  fi

  echo "EPHYR_RESTREAMER_STATE_PATH=${STATE_PATH}" >> "$ENV_FILE_PATH"
}

#################### SETUP ####################
CLEAR_EPHYR_STATE_ON_RESTART=${CLEAR_EPHYR_STATE_ON_RESTART:-0}
EPHYR_CLI_ARGS=${EPHYR_CLI_ARGS:-''}
EPHYR_VER=$(get_ephyr_version)
REGISTRY_URL=${REGISTRY_URL:-'docker.io'}
EPHYR_CONTAINER_NAME="ephyr-restreamer"
EPHYR_IMAGE_NAME="${REGISTRY_URL}/allatra/ephyr"
EPHYR_IMAGE_TAG="restreamer${EPHYR_VER}"

EPHYR_CONFIG_DIR="/var/lib/${EPHYR_CONTAINER_NAME}"
EPHYR_CONFIG_SRS_PATH="${EPHYR_CONFIG_DIR}/srs.conf"
EPHYR_CONFIG_STATE_PATH="${EPHYR_CONFIG_DIR}/state.json"
EPHYR_CONFIG_RUNTIME_ENV="${EPHYR_CONFIG_DIR}/ephyr-restreamer-env.list"
EPHYR_RESTREAMER_STATE_PATH="/tmp/workdir/state.json"

# Create required dir and files
mkdir -p $EPHYR_CONFIG_DIR
touch $EPHYR_CONFIG_STATE_PATH
touch $EPHYR_CONFIG_SRS_PATH
touch $EPHYR_CONFIG_RUNTIME_ENV


# The order of execution is matters here
update_and_install_docker
upgrade_if_required
login_to_registry_if_required "$REGISTRY_URL"
setup_firewall_rules

setup_runtime_config "$EPHYR_CONFIG_RUNTIME_ENV" "$EPHYR_RESTREAMER_STATE_PATH"



# Install Ephyr-restreamer executor
cat <<EOF > /usr/local/bin/run-ephyr-restreamer.sh
#!/usr/bin/env bash

set -e

clear_state_on_restart=$CLEAR_EPHYR_STATE_ON_RESTART
state_path=$EPHYR_CONFIG_STATE_PATH

# Detect directory for DVR.
ephyr_www_dir="/var/www/ephyr-restreamer"
do_volume="\$(set +e; find /mnt/volume_* -type d | head -1 | tr -d '\n')"
if [ -d "\$do_volume" ]; then
  ephyr_www_dir="\$do_volume/www"
fi
hcloud_volume="\$(set +e; find /mnt/HC_Volume_* -type d | head -1 | tr -d '\n')"
if [ -d "\$hcloud_volume" ]; then
  ephyr_www_dir="\$hcloud_volume/www"
fi

echo "ephyr_www_dir=\$ephyr_www_dir"
mkdir -p "\$ephyr_www_dir/"

# Print all required Environment variables.
echo "EPHYR_IMAGE_TAG=\$EPHYR_IMAGE_TAG"
echo "EPHYR_CLI_ARGS=\$EPHYR_CLI_ARGS"

if [[ \$clear_state_on_restart == "1" ]]; then
  rm \$state_path && touch \$state_path
fi

# Run Docker service
/usr/bin/docker run \
  --network=host \
  -v $EPHYR_CONFIG_SRS_PATH:/usr/local/srs/conf/srs.conf \
  -v \$state_path:$EPHYR_RESTREAMER_STATE_PATH \
  -v \$ephyr_www_dir/:/var/www/srs/ \
  --env-file $EPHYR_CONFIG_RUNTIME_ENV \
  --name=$EPHYR_CONTAINER_NAME \
  $EPHYR_IMAGE_NAME:\$EPHYR_IMAGE_TAG \$EPHYR_CLI_ARGS
EOF
chmod +x /usr/local/bin/run-ephyr-restreamer.sh


# Install Ephyr re-streamer SystemD Service.
cat <<EOF > /etc/systemd/system/ephyr-restreamer.service
[Unit]
Description=Ephyr service for re-streaming RTMP streams
After=local-fs.target docker.service
Requires=local-fs.target


[Service]
Environment=EPHYR_IMAGE_TAG=${EPHYR_IMAGE_TAG}
Environment=EPHYR_CLI_ARGS=${EPHYR_CLI_ARGS}

ExecStartPre=-/usr/bin/docker pull ${EPHYR_IMAGE_NAME}:\${EPHYR_IMAGE_TAG}
ExecStartPre=-/usr/bin/docker stop ${EPHYR_CONTAINER_NAME}
ExecStartPre=-/usr/bin/docker rm --volumes ${EPHYR_CONTAINER_NAME}
ExecStart=/usr/local/bin/run-ephyr-restreamer.sh
ExecStop=-/usr/bin/docker stop ${EPHYR_CONTAINER_NAME}
ExecStop=-/usr/bin/docker rm --volumes ${EPHYR_CONTAINER_NAME}

Restart=always


[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl unmask ephyr-restreamer.service
systemctl enable ephyr-restreamer.service
systemctl restart ephyr-restreamer.service
