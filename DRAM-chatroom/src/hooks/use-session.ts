/*
This file will act as a middle-man between our React UI and Rust backend
What it tracks (state):

The current session's id, name, participants
Connection status (connecting / connected / disconnected)
Any session errors (failed to join, kicked, etc.)

What it does (actions):

joinSession(id) — calls the Rust backend via Tauri invoke() to join, then triggers the view swap to SessionView
leaveSession() — tells the backend to disconnect, then swaps back to DashboardView
Listens for incoming Tauri events like incoming-message and updates state accordingly

You don't have to follow this structure exactly, but the idea is to centralize all session-related logic here so that our components can just call these functions and read from this state without worrying about the underlying implementation details.
*/
