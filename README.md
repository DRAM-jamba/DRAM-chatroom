# Quorthon

Private session-based chat

> Quorthon is under active development and is not production-ready.  

Quorthon is a self-hosted encrypted chat platform built with Rust and Tauri.  
It provides real-time messaging and voice communication without requiring traditional user accounts.

The app automatically creates a secure identity for each server you connect to, allowing you to join chat sessions without creating an account.


---

# Features

### Messaging

- Real-time messaging using WebSocket
- Session-based chat rooms
- Lightweight message protocol
- Client-side message encryption

### Security

- No account system
- Client-generated cryptographic identities

### Desktop Client

- Cross-platform desktop client built with **Tauri**
- Lightweight architecture
- Real-time UI updates via WebSocket

---

# How It Works

Anyone can run their own Quorthon server and clients can connect to any available one if they know the right address.

1. The user launches the Quorthon client
2. The client connects to a chosen Quorthon server
3. The client generates a secure local identity
4. The user joins or creates a chat session on that server
5. Messages and voice communication are exchanged in real time

---

# Installation for development

1. [Download and install required dependencies](https://tauri.app/start/prerequisites/)
2. Clone repository
`git clone https://github.com/DRAM-jamba/DRAM-chatroom.git`
`cd DRAM-chatroom`
3. Install frontend dependencies
`npm install`
4. Run development mode
`npm run tauri dev`

---

# License

MIT License
