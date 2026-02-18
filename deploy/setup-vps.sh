#!/usr/bin/env bash
# =============================================================================
# Knol — Hetzner VPS Initial Setup Script
# Run once on a fresh Hetzner CX32 (Ubuntu 22.04 / Debian 12)
# =============================================================================
# Usage:
#   ssh root@YOUR_VPS_IP
#   curl -fsSL https://raw.githubusercontent.com/aiknol/knol/main/deploy/setup-vps.sh | bash
# =============================================================================

set -euo pipefail

echo "========================================"
echo " Knol VPS Setup — Starter Tier"
echo "========================================"

# ---------- System updates ----------
echo "[1/7] Updating system packages..."
apt-get update -qq
apt-get upgrade -y -qq
apt-get install -y -qq \
    curl wget git ufw fail2ban unattended-upgrades \
    apt-transport-https ca-certificates gnupg lsb-release

# ---------- Create deploy user ----------
echo "[2/7] Creating deploy user..."
if ! id "knol" &>/dev/null; then
    useradd -m -s /bin/bash -G sudo knol
    echo "knol ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/knol
    echo "  → User 'knol' created. Add your SSH key:"
    echo "    ssh-copy-id knol@$(hostname -I | awk '{print $1}')"
fi

# ---------- Install Docker ----------
echo "[3/7] Installing Docker..."
if ! command -v docker &>/dev/null; then
    curl -fsSL https://get.docker.com | sh
    usermod -aG docker knol
    systemctl enable docker
    systemctl start docker
fi
echo "  → Docker $(docker --version | awk '{print $3}' | tr -d ',')"

# ---------- Install Docker Compose plugin ----------
echo "[4/7] Verifying Docker Compose..."
docker compose version &>/dev/null || {
    apt-get install -y docker-compose-plugin
}
echo "  → $(docker compose version)"

# ---------- Firewall ----------
echo "[5/7] Configuring firewall (UFW)..."
ufw --force reset
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
ufw allow 80/tcp    # HTTP
ufw allow 443/tcp   # HTTPS
ufw allow 443/udp   # HTTP/3
ufw --force enable
echo "  → UFW enabled (SSH + HTTP/S only)"

# ---------- Fail2ban ----------
echo "[6/7] Configuring fail2ban..."
systemctl enable fail2ban
systemctl start fail2ban

# ---------- Setup app directory ----------
echo "[7/7] Setting up application directory..."
APP_DIR=/opt/knol
mkdir -p $APP_DIR
chown knol:knol $APP_DIR

cat > $APP_DIR/README <<'EOF'
Knol Production Deployment
==========================

Directory structure:
  /opt/knol/
  ├── docker-compose.prod.yml   → Service definitions
  ├── Caddyfile                 → Reverse proxy config
  ├── .env.production           → Environment variables (DO NOT COMMIT)
  └── deploy.sh                 → Deploy/update script

Quick commands:
  cd /opt/knol
  docker compose -f docker-compose.prod.yml --env-file .env.production up -d      # Start
  docker compose -f docker-compose.prod.yml --env-file .env.production logs -f     # Logs
  docker compose -f docker-compose.prod.yml --env-file .env.production down        # Stop
  docker compose -f docker-compose.prod.yml --env-file .env.production pull        # Update images
EOF

chown knol:knol $APP_DIR/README

# ---------- Swap (recommended for 8GB VPS) ----------
echo "Setting up 2GB swap..."
if [ ! -f /swapfile ]; then
    fallocate -l 2G /swapfile
    chmod 600 /swapfile
    mkswap /swapfile
    swapon /swapfile
    echo '/swapfile none swap sw 0 0' >> /etc/fstab
    echo "vm.swappiness=10" >> /etc/sysctl.conf
    sysctl -p
fi

# ---------- Done ----------
echo ""
echo "========================================"
echo " VPS setup complete!"
echo "========================================"
echo ""
echo " Next steps:"
echo "  1. Add your SSH key:  ssh-copy-id knol@$(hostname -I | awk '{print $1}')"
echo "  2. Disable root SSH:  edit /etc/ssh/sshd_config → PermitRootLogin no"
echo "  3. Copy deploy files: scp deploy/* knol@VPS:/opt/knol/"
echo "  4. Create .env:       cp .env.production.example .env.production"
echo "  5. Point DNS:         api.aiknol.com → $(hostname -I | awk '{print $1}')"
echo "  6. Deploy:            docker compose -f docker-compose.prod.yml --env-file .env.production up -d"
echo ""
