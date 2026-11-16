---
name: telegram-bridge
description: >-
  Bi-directional Telegram conversation bridge for Antigravity. Listens for user
  commands via Telegram Bot, sends instant acknowledgments, action plans,
  periodic 1-2 minute progress updates, and task completions without button clutter.
---

# Telegram Bridge & Remote Conversation Workflow

This skill enforces a reliable, communicative, 4-stage feedback loop for all interactions between the user on Telegram and the Antigravity agent.

---

## 1. Mandatory 4-Stage Communication Protocol

When a user sends an instruction via Telegram, the agent and bridge must strictly follow this exact sequence:

```text
[User Sends Prompt]
        │
        ▼ (Stage 1: Automated Sub-Second Ack)
"⚡ Got the prompt, working on it! Will notify you shortly."
        │
        ▼ (Stage 2: Plan & Action Summary)
"📋 Action Plan: Here is what I am going to do:
 1. ...
 2. ...
 Starting execution now."
        │
        ▼ (Stage 3: Periodic Progress Updates - Every 1-2 Mins)
"🔄 Progress Update: Step 2 in progress... [Details]"
        │
        ▼ (Stage 4: Completion Summary)
"✅ Task Complete: [Summary of work done, files changed, verification results]"
```

---

## 2. Protocol Details

### Stage 1: Automatic Instant Acknowledgment
- Handled automatically at the transport layer by `telegram_cli.php wait-message`.
- As soon as the user's message is received, Telegram immediately replies:
  `⚡ Got the prompt, working on it! Will notify you shortly.`
- This reassures the user that the bridge received the prompt without delay.

### Stage 2: Action Plan Summary
- Before executing tool calls or heavy tasks, the agent sends a clean, bulleted summary of its planned actions:
  ```bash
  ./.agents/skills/telegram-bridge/scripts/telegram_cli.php send "📋 *Action Plan:*
  • Task: <Brief goal>
  • Steps to execute:
    1. <Step 1>
    2. <Step 2>
    3. <Step 3>
  Working on this now..."
  ```

### Stage 3: Persistent Progress Updates (Every 1–2 Minutes)
- During multi-step builds, compilation, audits, or downloads:
  ```bash
  ./.agents/skills/telegram-bridge/scripts/telegram_cli.php progress-update "🔨 *Progress Update (Step 2/3):*
  • Cross-compiling Windows PE32+ binary...
  • Time elapsed: 45s"
  ```
- Uses in-place message editing to avoid flooding the user's chat.

### Stage 4: Completion Delivery & Zero-Token Idle
- When work is finished, deliver a clean, comprehensive summary:
  ```bash
  ./.agents/skills/telegram-bridge/scripts/telegram_cli.php send "✅ *Task Completed Successfully!*
  <Summary of results>"
  ```
- **Return to Sleep**: Immediately re-launch the background listener:
  ```bash
  ./.agents/skills/telegram-bridge/scripts/listen_daemon.sh
  ```
  *(Set `WaitMsBeforeAsync: 500`)*. Consumes **0 tokens** while waiting for the next prompt.

### 🚫 No Unsolicited Inline Buttons Rule
- Do not attach arbitrary inline keyboard buttons unless explicitly requested by the user. Keep the interaction natural, communicative, and text-focused.
