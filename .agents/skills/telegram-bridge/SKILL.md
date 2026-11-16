---
name: telegram-bridge
description: >-
  Bi-directional Telegram conversation bridge for Antigravity. Listens for user
  commands via Telegram Bot, injects them into agent sessions, delivers persistent
  real-time progress updates (every 1-2 mins), provides interactive feedback
  controls (Queue, Stop, Status), and maintains zero token usage while idling.
---

# Telegram Bridge & Remote Agent Controller

This skill enables full remote control of the Antigravity agent through a Telegram Bot. It provides a zero-token idling listener, persistent live progress updates, and an interactive feedback loop with inline action buttons.

---

## 1. Zero-Token Idling Architecture

```text
[User in Telegram]
        │
        ▼ (Telegram Long Polling: getUpdates?timeout=45)
[telegram_cli.php wait-message]  <--- Running as a Background Task in Antigravity
        │                             (Uses 0 LLM tokens while waiting)
        ▼ (User hits Send)
[Task finishes with exit 0]
        │
        ▼ (Reactive Wakeup triggers Antigravity)
[Antigravity Context Resumes] ---> Reads command ---> Executes requested work
        │
        ▼ (Replies to User)
[telegram_cli.php send / progress-update] ---> [User receives formatted update in Telegram]
```

- While waiting for user commands, Antigravity **sleeps** and consumes **zero tokens**.
- As soon as the user sends a message or taps an inline button, the background task completes, waking up the agent with the user's exact message payload.

---

## 2. Configuration & Credentials

Telegram Bot credentials must strictly reside in `.telegram/config.json` (gitignored to comply with GitHub Security & Secret rules).

### Setup Command:
```bash
./.agents/skills/telegram-bridge/scripts/telegram_cli.php configure "<BOT_TOKEN>" "<ALLOWED_CHAT_ID>"
```

* **Security Whitelist**: The bridge automatically drops all messages from unknown Telegram IDs, ensuring only the authorized user can control the agent.

---

## 3. Operational Workflows for the Agent

### A. Entering Zero-Token Sleep to Wait for User Commands
When all current work is done and waiting for the user's next instruction via Telegram:
1. Run the background listener task:
   ```bash
   ./.agents/skills/telegram-bridge/scripts/telegram_cli.php wait-message 60
   ```
   *(Set `WaitMsBeforeAsync: 500` to background it immediately).*
2. End your turn without calling further tools.
3. The system will automatically wake you up when the user sends a message.

### B. Sending Persistent Progress Updates (Every 1–2 Minutes)
During long-running multi-step operations (e.g. compiling, downloading runtimes, configuring servers):
1. Send or edit the persistent status card:
   ```bash
   ./.agents/skills/telegram-bridge/scripts/telegram_cli.php progress-update "📦 Compiling Windows binary (Step 3/5)..."
   ```
2. The script edits the existing Telegram status message in-place using `editMessageText`, eliminating chat spam while keeping the user fully informed.

### C. Interactive Feedback Loop & Options Menu
If a task is actively running and the user sends an incoming command, offer interactive control buttons:
```bash
./.agents/skills/telegram-bridge/scripts/telegram_cli.php send-interactive \
  "⚠️ *Task in Progress*: Currently compiling the server.\nWhat would you like to do with your new command?" \
  '[[{"text":"➕ Queue Command","callback_data":"action:queue"},{"text":"⏹ Stop Current Task","callback_data":"action:stop"}],[{"text":"📊 View Logs & Status","callback_data":"action:status"}]]'
```

When the user taps a button, `wait-message` returns:
```json
{
  "type": "callback",
  "data": "action:queue",
  "message_id": 142,
  "chat_id": "12345678"
}
```
The agent reads `data` and acts accordingly:
- `action:queue`: Appends command to `.telegram/queue.json` and resumes the current task.
- `action:stop`: Kills running tasks and immediately begins the new command.
- `action:status`: Sends latest logs/progress snapshot.

### D. Sending Final Task Completion
When an operation finishes:
```bash
./.agents/skills/telegram-bridge/scripts/telegram_cli.php send "✅ *Task Completed Successfully!*\nNextcloud Server Windows build is ready."
```
