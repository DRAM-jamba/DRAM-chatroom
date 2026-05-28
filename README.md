# Quorthon

Quorthon is a self-hosted encrypted chat platform built with Rust and Tauri.  
Provides real-time messaging and voice communication without requiring traditional user accounts, 
instead the app automatically creates a secure identity for each server you connect to, allowing you to join chat sessions without creating an account.


---

# Features

### Messaging

- Real-time messaging over WebSocket
- Session-based chat rooms

### Voice

- In-session voice communication
- RNNoise noise suppression

### Security

- No account system — generates a cryptographic identity per server
- All traffic encrypted in transit over HTTPS and WSS

### Desktop Client

- Cross-platform desktop client built with Tauri v2 (Rust + TypeScript)
- Lightweight architecture
- Settings persistence and credential storage

---

# How It Works

1. Launch the Quorthon client
2. Connect to a Quorthon server by address
3. Pick a nickname and join or create a session
4. Chat and voice are exchanged in real time within the session

Anyone can run their own server. Clients can connect to any server they know the address of.

---

# Installation for development

1. Download and install required dependencies  
[Rust (stable)](https://rustup.rs/)  
[Node.js 20+](https://nodejs.org/)  
[Tauri v2 prerequisites](https://tauri.app/start/prerequisites/)
2. Setup  
```bash
git clone https://github.com/DRAM-jamba/DRAM-chatroom.git  
cd DRAM-chatroom/DRAM-client  
npm install
```
3. Run development mode  
```bash
npm run tauri dev
```
4. Build  
```bash
npm run tauri build
```
---

# Server Setup

### Requirements

- A Linux server with Docker and Docker Compose installed
- A domain name pointing to your server's IP (required for TLS)

### Steps

1. Install Docker
```bash
sudo apt install docker.io docker-compose-plugin   # Ubuntu/Debian
sudo dnf install docker docker-compose-plugin      # Fedora
sudo systemctl enable --now docker
```

2. Clone the repository
```bash
git clone https://github.com/DRAM-jamba/DRAM-chatroom.git
cd DRAM-chatroom/DRAM-server/server
```

3. Configure the server

Edit `docker-compose.yml` and set your own values for:
- `POSTGRES_PASSWORD` and `DATABASE_URL` — choose a strong database password
- `LIVEKIT_API_KEY` and `LIVEKIT_API_SECRET` — choose a strong key and secret
- `DOMAIN` — set your actual domain

Edit `livekit.yaml` and set the same key/secret:
```yaml
keys:
  your_key: "your_secret"
```

5. Start the server
```bash
docker compose build
docker compose up -d
```

Caddy will automatically obtain a TLS certificate from Let's Encrypt on first start.

6. Verify
```bash
curl https://yourdomain.com
# Should return: {"status":"ok","message":"server is running"}
```

Users can now connect to your server using your domain name in the Quorthon client.

---

# License

MIT License

---

# Third-party licenses

This project uses [RNNoise](https://github.com/xiph/rnnoise) via [@timephy/rnnoise-wasm](https://github.com/dumpvn/tim-rnnoise-wasm) for noise suppression.

RNNoise is licensed under BSD-3-Clause
