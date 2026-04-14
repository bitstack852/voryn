# Coolify Deployment Guide

Deploy Voryn infrastructure on your existing Coolify v4 instance.

## What Gets Deployed

| Service | Domain | Port | Purpose |
|---------|--------|------|---------|
| **Update Server** | updates.voryn.bitstack.website | 443 (HTTPS via Coolify) | Serves version.json + APK downloads |
| **Bootstrap Node** | boot1.voryn.bitstack.website | 4001 (raw TCP, direct) | DHT peer discovery for the P2P network |

---

## Step 1: Create the Project in Coolify

1. Log into your Coolify dashboard
2. Click **Projects** → **+ New**
3. Name: **Voryn**
4. Description: *Encrypted messaging infrastructure*
5. Click **Create**

---

## Step 2: Deploy the Update Server

This is the simpler service — pure nginx serving static files. Coolify handles TLS automatically.

### 2.1 Add a New Resource

1. Inside the **Voryn** project, click **+ New Resource**
2. Select **Docker Compose**
3. Choose **GitHub** (or paste the repo URL)
   - Repository: `bitstack852/voryn`
   - Branch: `claude/create-dev-plan-zBkvL` (or `main` after merge)
   - Docker Compose file path: `deploy/coolify/docker-compose.yml`
4. Click **Continue**

### 2.2 Configure the Update Server Service

After Coolify parses the compose file, you'll see both services. Configure `update-server`:

1. Click on the **update-server** service
2. **Domains:** `https://updates.voryn.bitstack.website`
3. **Port:** `80` (internal nginx port — Coolify's proxy handles 443→80)
4. **SSL:** Enabled (Coolify auto-generates Let's Encrypt cert)
5. Leave everything else default

### 2.3 Configure the Bootstrap Node Service

1. Click on the **bootstrap** service
2. **Domains:** Leave empty (this is not an HTTP service)
3. **Ports:** Make sure `4001:4001/tcp` and `4001:4001/udp` are mapped
   - In Coolify v4, go to the service's **Network** tab
   - Ensure the port mapping exposes 4001 directly on the host
   - This bypasses Coolify's Traefik proxy (libp2p is not HTTP)
4. **Persistent Storage:** Verify the `bootstrap-data` volume is mapped to `/data`

### 2.4 Deploy

Click **Deploy** in Coolify. Both containers will build and start.

---

## Step 3: Seed the Update Server Data

After the containers are running, populate version.json:

### Option A: Via Coolify Terminal

1. In Coolify, click the **update-server** service
2. Click **Terminal** (or **Execute Command**)
3. Run:
```bash
mkdir -p /usr/share/nginx/html/releases
cat > /usr/share/nginx/html/version.json << 'EOF'
{
  "latest": "0.1.0",
  "minimum": "0.1.0",
  "android_url": "https://updates.voryn.bitstack.website/releases/voryn-0.1.0-release.apk",
  "ios_url": "",
  "changelog": "Initial release",
  "released_at": "2026-04-14T00:00:00Z"
}
EOF
```

### Option B: Via Docker CLI on the Coolify Server

```bash
# SSH into your Coolify server
ssh admin@<COOLIFY_SERVER_IP>

# Find the container name
docker ps | grep update-server

# Exec into it
docker exec -it <CONTAINER_NAME> sh -c '
mkdir -p /usr/share/nginx/html/releases
cat > /usr/share/nginx/html/version.json << EOF
{
  "latest": "0.1.0",
  "minimum": "0.1.0",
  "android_url": "https://updates.voryn.bitstack.website/releases/voryn-0.1.0-release.apk",
  "ios_url": "",
  "changelog": "Initial release",
  "released_at": "2026-04-14T00:00:00Z"
}
EOF
'
```

---

## Step 4: DNS Setup in Cloudflare

In your Cloudflare dashboard for `bitstack.website`:

### Add these records:

| Type | Name | Content | Proxy Status | TTL |
|------|------|---------|-------------|-----|
| A | updates.voryn | `<COOLIFY_SERVER_IP>` | **Proxied** (orange cloud) | Auto |
| A | boot1.voryn | `<COOLIFY_SERVER_IP>` | **DNS only** (grey cloud) | Auto |

**Key difference:**
- **updates.voryn** → **Proxied** is fine because it's HTTPS traffic. Cloudflare adds DDoS protection and caching.
- **boot1.voryn** → Must be **DNS only** because port 4001 TCP/UDP doesn't go through Cloudflare's proxy.

### Cloudflare SSL Setting

If you're proxying the update server through Cloudflare:
1. Go to SSL/TLS → Overview
2. Set to **Full (strict)** — Coolify's Let's Encrypt cert validates end-to-end

---

## Step 5: Verify Deployment

### Update Server

```bash
# Check version.json is accessible
curl -sf https://updates.voryn.bitstack.website/version.json | jq .

# Expected:
# {
#   "latest": "0.1.0",
#   "minimum": "0.1.0",
#   ...
# }

# Check health endpoint
curl -sf https://updates.voryn.bitstack.website/health | jq .

# Expected: {"status":"ok"}
```

### Bootstrap Node

```bash
# Check port 4001 is reachable
nc -zv boot1.voryn.bitstack.website 4001

# Expected: Connection to boot1.voryn.bitstack.website 4001 port [tcp/*] succeeded!
```

### From Coolify Dashboard

1. Both services should show **Running** (green) status
2. Check **Logs** for each service for any errors

---

## Step 6: Set Up Auto-Deploy (Optional)

Coolify v4 supports webhook-triggered deploys from GitHub:

1. In Coolify, go to the Voryn project → Settings
2. Enable **Auto Deploy** on push
3. Copy the webhook URL
4. In GitHub: repo → Settings → Webhooks → Add webhook
   - Payload URL: `<Coolify webhook URL>`
   - Content type: `application/json`
   - Events: Just the push event
   - Branch filter: `main`

Now pushing to `main` automatically redeploys both services.

---

## Uploading App Releases

When you have a signed APK to release:

### Manual Upload

```bash
# SSH into the Coolify server
ssh admin@<COOLIFY_SERVER_IP>

# Find the update-server container
CONTAINER=$(docker ps --format '{{.Names}}' | grep update-server)

# Copy APK into the container
docker cp voryn-0.2.0-release.apk ${CONTAINER}:/usr/share/nginx/html/releases/
docker cp voryn-0.2.0-release.apk.sha256 ${CONTAINER}:/usr/share/nginx/html/releases/

# Update version.json
docker exec ${CONTAINER} sh -c 'cat > /usr/share/nginx/html/version.json << EOF
{
  "latest": "0.2.0",
  "minimum": "0.1.0",
  "android_url": "https://updates.voryn.bitstack.website/releases/voryn-0.2.0-release.apk",
  "ios_url": "",
  "changelog": "New features and bug fixes",
  "released_at": "2026-04-15T00:00:00Z"
}
EOF'

# Verify
curl -sf https://updates.voryn.bitstack.website/version.json | jq .latest
# Expected: "0.2.0"
```

### Via CI/CD (GitHub Actions)

The `.github/workflows/release.yml` workflow can deploy automatically. You'll need to:

1. Add an SSH key for the Coolify server as a GitHub secret (`COOLIFY_SSH_KEY`)
2. Modify the deploy step to `docker cp` instead of `scp` to the nginx web root

---

## Troubleshooting

### Update server returns 502/503

- Check Coolify logs for the update-server container
- Verify nginx is running: `docker exec <container> nginx -t`
- Check the domain is pointed correctly: `dig updates.voryn.bitstack.website`

### Bootstrap node port 4001 not reachable

- Verify the port is exposed on the host: `docker port <bootstrap-container>`
- Check Coolify's port mapping — 4001 must be mapped to the host, not proxied through Traefik
- Check the Coolify server's firewall: `sudo ufw status` (must allow 4001 TCP/UDP)
- If using Hyper-V port forwarding, verify the forwarding rules on the Windows host

### version.json is empty or 404

- The `update-data` volume needs to be seeded (Step 3)
- If the container was recreated, the data may be lost — reseed it
- Consider mounting a host directory instead of a Docker volume for persistence

### TLS certificate not working

- If using Cloudflare proxy (orange cloud): set SSL mode to **Full (strict)**
- If DNS only (grey cloud): Coolify handles the cert — check Coolify's SSL settings
- Run `docker exec <container> certbot certificates` if using standalone certbot

---

## Architecture on Coolify

```
Cloudflare DNS
  │
  ├── updates.voryn.bitstack.website (Proxied → orange cloud)
  │     │
  │     └── Coolify Traefik Proxy (:443 TLS termination)
  │           │
  │           └── update-server container (nginx :80)
  │                 └── /version.json, /releases/*.apk
  │
  └── boot1.voryn.bitstack.website (DNS only → grey cloud)
        │
        └── Direct to Coolify server :4001
              │
              └── bootstrap container (libp2p :4001 TCP/UDP)
                    └── Kademlia DHT peer discovery
```
