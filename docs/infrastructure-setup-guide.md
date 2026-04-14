# Infrastructure Setup Guide — Hyper-V on Windows Server 2019

This guide walks through setting up Voryn infrastructure on a single Ubuntu 22.04 VM
running on Hyper-V, hosted on a Gigabyte G292-Z20 server with AMD EPYC 7002.

## What We're Setting Up

```
Windows Server 2019 (Hyper-V Host)
  └── VM: "voryn-infra" (Ubuntu 22.04 LTS)
        ├── Voryn Bootstrap Node (port 4001 TCP/UDP)
        │     └── DHT peer discovery for the P2P network
        └── Voryn Update Server (port 443 HTTPS)
              └── nginx serving version.json + APK downloads
```

**Resources needed:** 2 vCPU, 2GB RAM, 40GB disk (very light workload)

---

## Step 1: Create the Ubuntu VM in Hyper-V

### 1.1 Download Ubuntu 22.04 Server ISO

On your Windows Server, download the ISO:
```
https://releases.ubuntu.com/22.04/ubuntu-22.04.4-live-server-amd64.iso
```

Save it somewhere accessible, e.g. `D:\ISOs\ubuntu-22.04.4-live-server-amd64.iso`

### 1.2 Create the VM via Hyper-V Manager

Open **Hyper-V Manager** and follow these steps:

1. Right-click your server name → **New** → **Virtual Machine**
2. **Name:** `voryn-infra`
3. **Generation:** Generation 2 (UEFI)
4. **Memory:** 2048 MB (uncheck "Use Dynamic Memory" for consistency)
5. **Networking:** Select the virtual switch connected to your public network
6. **Virtual Hard Disk:** Create new, 40 GB, default location is fine
7. **Installation:** Select "Install from ISO" → browse to the Ubuntu ISO
8. Click **Finish**

### 1.3 Configure VM Settings Before First Boot

Right-click `voryn-infra` → **Settings**:

- **Security:** Uncheck "Enable Secure Boot" (or set template to "Microsoft UEFI Certificate Authority" — Ubuntu needs this changed)
- **Processor:** 2 virtual processors
- **Integration Services:** Enable all (Guest services, Heartbeat, etc.)
- **Checkpoints:** Disable automatic checkpoints (we'll use manual ones)

### 1.4 Install Ubuntu

1. Right-click `voryn-infra` → **Connect** → **Start**
2. Ubuntu installer will boot. Select:
   - Language: English
   - Keyboard: Your preference
   - Installation type: **Ubuntu Server (minimized)**
   - Network: Configure the static IP for this VM (see Step 2)
   - Storage: Use entire disk, no LVM needed
   - **Your name:** `admin`
   - **Server name:** `voryn-infra`
   - **Username:** `admin`
   - **Password:** Choose a strong password
   - **SSH:** Install OpenSSH server ✓
   - **Snaps:** Don't select any
3. Wait for installation to complete, then reboot
4. **Remove the ISO:** In Hyper-V Settings → DVD Drive → None

### 1.5 First Boot — Verify Access

```powershell
# From your Windows Server or local machine
ssh admin@<VM_IP_ADDRESS>
```

---

## Step 2: Assign a Public IP to the VM

Since you have a spare public IP, you'll assign it directly to the VM.

### Option A: Static IP on the VM (if your network switch bridges to the public network)

Edit the netplan config on the Ubuntu VM:

```bash
sudo nano /etc/netplan/00-installer-config.yaml
```

Replace the contents with (adjust IPs to match your network):

```yaml
network:
  version: 2
  ethernets:
    eth0:
      addresses:
        - <YOUR_PUBLIC_IP>/24        # e.g. 203.0.113.50/24
      routes:
        - to: default
          via: <YOUR_GATEWAY_IP>     # e.g. 203.0.113.1
      nameservers:
        addresses:
          - 1.1.1.1
          - 8.8.8.8
```

Apply:
```bash
sudo netplan apply
```

Verify:
```bash
# Check IP assigned
ip addr show eth0

# Check internet access
ping -c 3 1.1.1.1

# Check DNS
ping -c 3 google.com
```

### Option B: NAT with Port Forwarding (if VMs share the host's IP)

If VMs are behind NAT on the Hyper-V host, forward these ports to the VM:

| External Port | Internal (VM) Port | Protocol | Purpose |
|--------------|-------------------|----------|---------|
| 4001 | 4001 | TCP | libp2p bootstrap node |
| 4001 | 4001 | UDP | libp2p QUIC transport |
| 80 | 80 | TCP | HTTP (certbot + redirect) |
| 443 | 443 | TCP | HTTPS (update server) |

In PowerShell on the Hyper-V host:
```powershell
# Forward port 4001 TCP
netsh interface portproxy add v4tov4 listenport=4001 listenaddress=0.0.0.0 connectport=4001 connectaddress=<VM_INTERNAL_IP>

# Forward port 443
netsh interface portproxy add v4tov4 listenport=443 listenaddress=0.0.0.0 connectport=443 connectaddress=<VM_INTERNAL_IP>

# Forward port 80
netsh interface portproxy add v4tov4 listenport=80 listenaddress=0.0.0.0 connectport=80 connectaddress=<VM_INTERNAL_IP>

# Also open Windows Firewall
netsh advfirewall firewall add rule name="Voryn Bootstrap TCP" dir=in action=allow protocol=TCP localport=4001
netsh advfirewall firewall add rule name="Voryn Bootstrap UDP" dir=in action=allow protocol=UDP localport=4001
netsh advfirewall firewall add rule name="Voryn HTTPS" dir=in action=allow protocol=TCP localport=443
netsh advfirewall firewall add rule name="Voryn HTTP" dir=in action=allow protocol=TCP localport=80
```

Verify from outside your network:
```bash
# From any external machine
nc -zv <PUBLIC_IP> 4001
nc -zv <PUBLIC_IP> 443
```

---

## Step 3: Secure the Ubuntu VM

SSH into the VM and run these commands:

```bash
# Update everything
sudo apt update && sudo apt upgrade -y

# Install essential packages
sudo apt install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    ufw \
    fail2ban \
    unattended-upgrades \
    nginx \
    certbot \
    python3-certbot-nginx \
    jq

# ── Firewall ─────────────────────────────────────────
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 4001/tcp comment "Voryn libp2p TCP"
sudo ufw allow 4001/udp comment "Voryn libp2p QUIC"
sudo ufw allow 80/tcp comment "HTTP (certbot)"
sudo ufw allow 443/tcp comment "HTTPS (update server)"
sudo ufw --force enable
sudo ufw status

# ── SSH Hardening ────────────────────────────────────
# Disable password auth (make sure your SSH key works first!)
sudo sed -i 's/#PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
sudo sed -i 's/PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
sudo systemctl restart sshd

# ── Automatic Security Updates ───────────────────────
sudo dpkg-reconfigure -plow unattended-upgrades
# Select "Yes" when prompted

# ── Fail2ban (brute force protection) ────────────────
sudo systemctl enable --now fail2ban
```

---

## Step 4: Install Rust

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Verify
rustc --version
cargo --version
```

---

## Step 5: Build and Run the Bootstrap Node

### 5.1 Create the voryn user

```bash
sudo useradd -r -m -d /opt/voryn -s /bin/false voryn
sudo mkdir -p /opt/voryn/{bin,data,src}
sudo chown -R voryn:voryn /opt/voryn
```

### 5.2 Clone and build

```bash
cd /opt/voryn/src
sudo -u voryn git clone https://github.com/bitstack852/voryn.git .
sudo -u voryn bash -c 'source /home/admin/.cargo/env && cargo build --release -p voryn-network'
```

> **Note:** The bootstrap binary (`voryn-bootstrap`) doesn't exist as a standalone binary yet — 
> it will be created during Phase 1 production implementation. For now, verify the library compiles.
> Once the binary exists, copy it:
> ```bash
> sudo cp /opt/voryn/src/target/release/voryn-bootstrap /opt/voryn/bin/
> ```

### 5.3 Install the systemd service

```bash
sudo tee /etc/systemd/system/voryn-bootstrap.service > /dev/null << 'EOF'
[Unit]
Description=Voryn DHT Bootstrap Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=voryn
Group=voryn
WorkingDirectory=/opt/voryn
ExecStart=/opt/voryn/bin/voryn-bootstrap --listen /ip4/0.0.0.0/tcp/4001 --identity-file /opt/voryn/data/node-identity.key --log-level info
Restart=always
RestartSec=5
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/opt/voryn/data
PrivateTmp=yes
LimitNOFILE=65536
MemoryMax=256M

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
# Enable and start once the binary is built:
# sudo systemctl enable --now voryn-bootstrap
# sudo systemctl status voryn-bootstrap
# journalctl -u voryn-bootstrap -f
```

---

## Step 6: Set Up the Update Server (nginx)

### 6.1 Create the web directory

```bash
sudo mkdir -p /var/www/voryn-updates/releases
sudo chown -R www-data:www-data /var/www/voryn-updates
```

### 6.2 Seed version.json

```bash
sudo tee /var/www/voryn-updates/version.json > /dev/null << 'EOF'
{
  "latest": "0.1.0",
  "minimum": "0.1.0",
  "android_url": "",
  "ios_url": "",
  "changelog": "Initial release - infrastructure setup",
  "released_at": "2026-04-14T00:00:00Z"
}
EOF
```

### 6.3 Configure nginx

```bash
sudo tee /etc/nginx/sites-available/voryn-updates > /dev/null << 'NGINX'
server {
    listen 80;
    listen [::]:80;
    server_name updates.YOUR_DOMAIN.com;

    root /var/www/voryn-updates;

    # version.json — short cache
    location = /version.json {
        expires 5m;
        add_header Cache-Control "public, max-age=300";
        add_header Access-Control-Allow-Origin "*" always;
    }

    # APK releases — long cache (immutable files)
    location /releases/ {
        expires 365d;
        add_header Cache-Control "public, max-age=31536000, immutable";
        add_header Access-Control-Allow-Origin "*" always;
    }

    location / {
        return 404;
    }
}
NGINX

# Enable the site
sudo ln -sf /etc/nginx/sites-available/voryn-updates /etc/nginx/sites-enabled/
sudo rm -f /etc/nginx/sites-enabled/default

# Test and reload
sudo nginx -t
sudo systemctl reload nginx
```

### 6.4 Verify HTTP works

```bash
# From the VM itself
curl -s http://localhost/version.json | jq .

# From outside (use the public IP)
curl -s http://<PUBLIC_IP>/version.json | jq .
```

---

## Step 7: Set Up DNS (Cloudflare)

In your Cloudflare dashboard for your domain:

### Add these DNS records:

| Type | Name | Content | Proxy | TTL |
|------|------|---------|-------|-----|
| A | boot1 | `<VM_PUBLIC_IP>` | **DNS only** (grey cloud) | Auto |
| A | updates | `<VM_PUBLIC_IP>` | **DNS only** (grey cloud) | Auto |

**Important:** Set the proxy to **DNS only** (grey cloud icon), not "Proxied" (orange cloud).
The bootstrap node uses raw TCP on port 4001 which Cloudflare's proxy doesn't support.
For the update server, DNS-only keeps things simple with Let's Encrypt.

### Verify DNS propagation:

```bash
# Wait a minute, then check
dig +short boot1.YOUR_DOMAIN.com
dig +short updates.YOUR_DOMAIN.com
# Both should return your VM's public IP
```

---

## Step 8: Enable HTTPS (Let's Encrypt)

Once DNS is pointing to your VM:

```bash
sudo certbot --nginx -d updates.YOUR_DOMAIN.com

# Follow the prompts:
# - Enter email for renewal notices
# - Agree to terms
# - Select "Redirect HTTP to HTTPS" when asked

# Verify auto-renewal
sudo certbot renew --dry-run

# Test HTTPS
curl -sf https://updates.YOUR_DOMAIN.com/version.json | jq .
```

---

## Step 9: Create Deploy User (for CI/CD)

```bash
# Create the deploy user
sudo useradd -r -m -s /bin/bash voryn-deploy
sudo usermod -aG www-data voryn-deploy

# Allow deploy user to write releases
sudo chmod 775 /var/www/voryn-updates /var/www/voryn-updates/releases

# Set up SSH key auth for the deploy user
sudo mkdir -p /home/voryn-deploy/.ssh
sudo chmod 700 /home/voryn-deploy/.ssh

# Generate a deploy key (run this on YOUR local machine, not the server)
# ssh-keygen -t ed25519 -f voryn-deploy-key -C "voryn-ci-deploy"
# Then copy the PUBLIC key to the server:

# On the server, paste the public key:
sudo tee /home/voryn-deploy/.ssh/authorized_keys << 'EOF'
# PASTE YOUR PUBLIC KEY HERE
# e.g.: ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIxxxxxx voryn-ci-deploy
EOF

sudo chmod 600 /home/voryn-deploy/.ssh/authorized_keys
sudo chown -R voryn-deploy:voryn-deploy /home/voryn-deploy/.ssh
```

---

## Step 10: Verify Everything

Run this checklist from an **external machine** (not the VM itself):

```bash
echo "=== Voryn Infrastructure Verification ==="

# 1. Bootstrap node port
echo -n "Bootstrap TCP 4001: "
nc -zv <PUBLIC_IP> 4001 2>&1 && echo "OK" || echo "FAIL"

# 2. Update server HTTPS
echo -n "Update server HTTPS: "
curl -sf https://updates.YOUR_DOMAIN.com/version.json > /dev/null && echo "OK" || echo "FAIL"

# 3. version.json content
echo -n "version.json valid: "
curl -sf https://updates.YOUR_DOMAIN.com/version.json | jq -e .latest > /dev/null && echo "OK" || echo "FAIL"

# 4. DNS resolution
echo -n "boot1 DNS: "
dig +short boot1.YOUR_DOMAIN.com | head -1

echo -n "updates DNS: "
dig +short updates.YOUR_DOMAIN.com | head -1

# 5. SSH deploy access (from CI or local machine with deploy key)
echo -n "Deploy SSH: "
ssh -i voryn-deploy-key -o ConnectTimeout=5 voryn-deploy@updates.YOUR_DOMAIN.com "echo OK" 2>/dev/null || echo "FAIL"
```

Expected output:
```
Bootstrap TCP 4001: OK        (or FAIL until bootstrap binary is built)
Update server HTTPS: OK
version.json valid: OK
boot1 DNS: 203.0.113.50
updates DNS: 203.0.113.50
Deploy SSH: OK
```

---

## Step 11: Take a Hyper-V Checkpoint

Now that everything is configured, create a checkpoint so you can roll back if needed:

In **Hyper-V Manager:**
1. Right-click `voryn-infra` → **Checkpoint**
2. Name it: `voryn-infra-base-setup-2026-04-14`

---

## Quick Reference

| Component | Address | Port | Status Command |
|-----------|---------|------|---------------|
| Bootstrap Node | boot1.YOUR_DOMAIN.com | 4001 | `sudo systemctl status voryn-bootstrap` |
| Update Server | updates.YOUR_DOMAIN.com | 443 | `sudo systemctl status nginx` |
| SSH (admin) | `<PUBLIC_IP>` | 22 | `ssh admin@<IP>` |
| SSH (deploy) | `<PUBLIC_IP>` | 22 | `ssh -i deploy-key voryn-deploy@<IP>` |
| Firewall | — | — | `sudo ufw status` |
| TLS Cert | — | — | `sudo certbot certificates` |
| Logs (bootstrap) | — | — | `journalctl -u voryn-bootstrap -f` |
| Logs (nginx) | — | — | `tail -f /var/log/nginx/access.log` |

---

## Next Steps After Infrastructure Is Up

1. **Add GitHub Secrets** — Add the deploy SSH private key as `UPDATE_SERVER_SSH_KEY` in your repo settings
2. **Generate Android keystore** — Run the `keytool` command from the deployment plan and add as GitHub secrets
3. **Build the bootstrap binary** — Once Phase 1 code is production-ready, build and install it
4. **First release** — Tag `v0.1.0`, let CI/CD build and deploy automatically
5. **Add monitoring** — Set up UptimeRobot checks for port 4001 and the HTTPS endpoint
