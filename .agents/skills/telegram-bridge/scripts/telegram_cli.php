#!/usr/bin/env php
<?php
/**
 * Telegram Bridge CLI for Antigravity Agent
 * High-performance, zero-token-idle bi-directional communication bridge.
 * Features: Long-polling listener, persistent status updates, inline feedback buttons.
 */

declare(strict_types=1);

$BASE_DIR = dirname(__DIR__, 4); // repo root: /mnt/data/roufsweb/nextcloud-server-windows
$CONFIG_DIR = $BASE_DIR . '/.telegram';
$CONFIG_FILE = $CONFIG_DIR . '/config.json';
$STATE_FILE = $CONFIG_DIR . '/state.json';
$QUEUE_FILE = $CONFIG_DIR . '/queue.json';

// Ensure config directory exists
if (!is_dir($CONFIG_DIR)) {
    @mkdir($CONFIG_DIR, 0700, true);
}

function load_config(): array {
    global $CONFIG_FILE;
    if (file_exists($CONFIG_FILE)) {
        $data = json_decode((string)file_get_contents($CONFIG_FILE), true);
        if (is_array($data)) return $data;
    }
    return [
        'bot_token' => getenv('TELEGRAM_BOT_TOKEN') ?: '',
        'allowed_username' => getenv('TELEGRAM_ALLOWED_USER') ?: 'roufsweb',
        'allowed_chat_id' => getenv('TELEGRAM_CHAT_ID') ?: '',
    ];
}

function save_config(array $cfg): void {
    global $CONFIG_FILE;
    file_put_contents($CONFIG_FILE, json_encode($cfg, JSON_PRETTY_PRINT));
    @chmod($CONFIG_FILE, 0600);
}

function load_state(): array {
    global $STATE_FILE;
    if (file_exists($STATE_FILE)) {
        $data = json_decode((string)file_get_contents($STATE_FILE), true);
        if (is_array($data)) return $data;
    }
    return ['last_update_id' => 0, 'last_status_message_id' => null];
}

function save_state(array $state): void {
    global $STATE_FILE;
    file_put_contents($STATE_FILE, json_encode($state, JSON_PRETTY_PRINT));
}

function telegram_api(string $method, array $params = []): ?array {
    $config = load_config();
    $token = $config['bot_token'];
    if (empty($token)) {
        fwrite(STDERR, "[ERROR] Telegram bot token not configured. Run 'php telegram_cli.php configure' first.\n");
        return null;
    }

    $url = "https://api.telegram.org/bot{$token}/{$method}";
    $ch = curl_init($url);
    curl_setopt_array($ch, [
        CURLOPT_RETURNTRANSFER => true,
        CURLOPT_POST           => true,
        CURLOPT_POSTFIELDS     => json_encode($params),
        CURLOPT_HTTPHEADER     => ['Content-Type: application/json'],
        CURLOPT_TIMEOUT        => isset($params['timeout']) ? ($params['timeout'] + 10) : 30,
        CURLOPT_SSL_VERIFYPEER => true,
    ]);

    $response = curl_exec($ch);
    $httpCode = curl_getinfo($ch, CURLINFO_HTTP_CODE);
    $err = curl_error($ch);
    curl_close($ch);

    if ($response === false || $httpCode >= 400) {
        fwrite(STDERR, "[ERROR] Telegram API request failed: HTTP {$httpCode} - {$err} - {$response}\n");
        return null;
    }

    $decoded = json_decode($response, true);
    return is_array($decoded) && ($decoded['ok'] ?? false) ? $decoded['result'] : null;
}

// -----------------------------------------------------------------------------
// CLI Subcommands
// -----------------------------------------------------------------------------
$cmd = $argv[1] ?? 'help';

switch ($cmd) {
    case 'configure':
        $token = $argv[2] ?? null;
        $userOrChatId = $argv[3] ?? null;

        if (!$token || !$userOrChatId) {
            echo "Usage: php telegram_cli.php configure <BOT_TOKEN> <USERNAME_OR_CHAT_ID>\n";
            exit(1);
        }

        $cleanUser = ltrim(trim($userOrChatId), '@');
        $isNumeric = is_numeric($cleanUser);

        $cfg = [
            'bot_token' => trim($token),
            'allowed_username' => $isNumeric ? '' : strtolower($cleanUser),
            'allowed_chat_id' => $isNumeric ? $cleanUser : '',
        ];
        save_config($cfg);
        echo "✓ Telegram configuration saved successfully in .telegram/config.json\n";
        echo "  Bot Token: " . substr(trim($token), 0, 10) . "...\n";
        echo "  Allowed User: " . ($cfg['allowed_username'] ? '@' . $cfg['allowed_username'] : 'ID: ' . $cfg['allowed_chat_id']) . "\n";
        break;

    case 'send':
        $text = $argv[2] ?? '';
        if (empty($text)) {
            fwrite(STDERR, "Error: Missing message text.\n");
            exit(1);
        }
        $config = load_config();
        $chatId = $config['allowed_chat_id'];
        $res = telegram_api('sendMessage', [
            'chat_id'    => $chatId,
            'text'       => $text,
            'parse_mode' => 'Markdown',
            'disable_web_page_preview' => true,
        ]);
        // Fallback to plain text if markdown formatting failed
        if (!$res) {
            $res = telegram_api('sendMessage', [
                'chat_id'    => $chatId,
                'text'       => $text,
                'disable_web_page_preview' => true,
            ]);
        }
        if ($res && isset($res['message_id'])) {
            echo json_encode(['status' => 'sent', 'message_id' => $res['message_id']]) . "\n";
        }
        break;

    case 'send-photo':
        $photoPath = $argv[2] ?? '';
        $caption = $argv[3] ?? '';
        if (!file_exists($photoPath)) {
            fwrite(STDERR, "Error: Photo file does not exist at '{$photoPath}'.\n");
            exit(1);
        }
        $config = load_config();
        $token = $config['bot_token'];
        $chatId = $config['allowed_chat_id'];

        $ch = curl_init("https://api.telegram.org/bot{$token}/sendPhoto");
        curl_setopt_array($ch, [
            CURLOPT_POST => true,
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_POSTFIELDS => [
                'chat_id' => $chatId,
                'photo' => new CURLFile($photoPath),
                'caption' => $caption,
                'parse_mode' => 'Markdown'
            ]
        ]);
        $response = curl_exec($ch);
        curl_close($ch);
        echo $response . "\n";
        break;

    case 'send-interactive':
        // Usage: php telegram_cli.php send-interactive "Task is running..." '[["Queue","queue"],["Stop","stop"]]'
        $text = $argv[2] ?? '';
        $optionsRaw = $argv[3] ?? '[]';
        $options = json_decode($optionsRaw, true) ?: [
            [['text' => '➕ Add to Queue', 'callback_data' => 'action:queue'], ['text' => '⏹ Stop Task', 'callback_data' => 'action:stop']],
            [['text' => '📊 View Status', 'callback_data' => 'action:status']]
        ];

        $config = load_config();
        $chatId = $config['allowed_chat_id'];
        $res = telegram_api('sendMessage', [
            'chat_id'      => $chatId,
            'text'         => $text,
            'parse_mode'   => 'Markdown',
            'reply_markup' => ['inline_keyboard' => $options],
        ]);
        if ($res && isset($res['message_id'])) {
            echo json_encode(['status' => 'sent', 'message_id' => $res['message_id']]) . "\n";
        }
        break;

    case 'progress-update':
        // Usage: php telegram_cli.php progress-update "Progress: 50%" [optional_message_id]
        $text = $argv[2] ?? '';
        $msgId = isset($argv[3]) ? (int)$argv[3] : null;
        $config = load_config();
        $state = load_state();
        $chatId = $config['allowed_chat_id'];

        if (!$msgId && !empty($state['last_status_message_id'])) {
            $msgId = (int)$state['last_status_message_id'];
        }

        $formatted = "🔄 *Agent Progress Update*\n" . $text . "\n\n_Updated: " . date('H:i:s UTC') . "_";

        $res = null;
        if ($msgId) {
            // Attempt to edit existing message
            $res = telegram_api('editMessageText', [
                'chat_id'    => $chatId,
                'message_id' => $msgId,
                'text'       => $formatted,
                'parse_mode' => 'Markdown',
            ]);
        }

        if (!$res) {
            // Send new message if edit fails or no message ID
            $res = telegram_api('sendMessage', [
                'chat_id'    => $chatId,
                'text'       => $formatted,
                'parse_mode' => 'Markdown',
            ]);
            if (!$res) {
                $res = telegram_api('sendMessage', [
                    'chat_id'    => $chatId,
                    'text'       => $formatted,
                ]);
            }
        }

        if ($res && isset($res['message_id'])) {
            $state['last_status_message_id'] = $res['message_id'];
            save_state($state);
            echo json_encode(['status' => 'updated', 'message_id' => $res['message_id']]) . "\n";
        }
        break;

    case 'wait-message':
        // Long poll for incoming message or button callback. Blocks until message arrives.
        $timeout = isset($argv[2]) ? (int)$argv[2] : 45;
        $config = load_config();
        $state = load_state();
        $allowedChatId = (string)($config['allowed_chat_id'] ?? '');
        $allowedUser = strtolower(ltrim((string)($config['allowed_username'] ?? ''), '@'));

        if (empty($config['bot_token']) || (empty($allowedChatId) && empty($allowedUser))) {
            fwrite(STDERR, "[ERROR] Missing credentials in .telegram/config.json\n");
            exit(1);
        }

        $offset = $state['last_update_id'] + 1;

        $updates = telegram_api('getUpdates', [
            'offset'  => $offset,
            'timeout' => $timeout,
            'allowed_updates' => ['message', 'callback_query'],
        ]);

        if (is_array($updates) && count($updates) > 0) {
            foreach ($updates as $update) {
                $updateId = $update['update_id'];
                if ($updateId > $state['last_update_id']) {
                    $state['last_update_id'] = $updateId;
                }

                // Check authorization helper
                $checkAuth = function(array $entity) use (&$config, $allowedChatId, $allowedUser): bool {
                    $senderId = (string)($entity['chat']['id'] ?? $entity['from']['id'] ?? '');
                    $senderUser = strtolower(ltrim((string)($entity['from']['username'] ?? $entity['chat']['username'] ?? ''), '@'));

                    if (!empty($allowedUser) && $senderUser === $allowedUser) {
                        if (empty($config['allowed_chat_id']) || $config['allowed_chat_id'] !== $senderId) {
                            $config['allowed_chat_id'] = $senderId;
                            save_config($config);
                        }
                        return true;
                    }
                    if (!empty($allowedChatId) && $senderId === $allowedChatId) {
                        return true;
                    }
                    return false;
                };

                // Handle Callback Query (Button Press)
                if (isset($update['callback_query'])) {
                    $cb = $update['callback_query'];
                    $senderId = (string)($cb['message']['chat']['id'] ?? $cb['from']['id'] ?? '');

                    telegram_api('answerCallbackQuery', ['callback_query_id' => $cb['id']]);

                    if ($checkAuth($cb)) {
                        save_state($state);
                        echo json_encode([
                            'type'        => 'callback',
                            'data'        => $cb['data'] ?? '',
                            'message_id'  => $cb['message']['message_id'] ?? null,
                            'chat_id'     => $senderId,
                            'timestamp'   => time(),
                        ], JSON_UNESCAPED_SLASHES) . "\n";
                        exit(0);
                    }
                }

                // Handle Text Message
                if (isset($update['message'])) {
                    $msg = $update['message'];
                    $senderId = (string)($msg['chat']['id'] ?? '');
                    $text = $msg['text'] ?? '';

                    if ($checkAuth($msg) && !empty($text)) {
                        save_state($state);

                        // Immediate acknowledgment receipt back to Telegram
                        $escapedPreview = htmlspecialchars(mb_strimwidth($text, 0, 70, '...'), ENT_QUOTES, 'UTF-8');
                        telegram_api('sendMessage', [
                            'chat_id'    => $senderId,
                            'text'       => "⚡ <b>Got the prompt, working on it!</b>\n<i>\"{$escapedPreview}\"</i>\nWill notify you shortly.",
                            'parse_mode' => 'HTML',
                        ]);

                        echo json_encode([
                            'type'        => 'message',
                            'text'        => $text,
                            'message_id'  => $msg['message_id'] ?? null,
                            'chat_id'     => $senderId,
                            'timestamp'   => $msg['date'] ?? time(),
                        ], JSON_UNESCAPED_SLASHES) . "\n";
                        exit(0);
                    }
                }
            }
            save_state($state);
        }

        // Return timeout if no authorized message arrived during the polling window
        echo json_encode(['type' => 'timeout', 'timestamp' => time()]) . "\n";
        exit(0);
        break;

    case 'status':
        $config = load_config();
        $state = load_state();
        echo json_encode([
            'configured'       => !empty($config['bot_token']) && (!empty($config['allowed_chat_id']) || !empty($config['allowed_username'])),
            'allowed_username' => $config['allowed_username'] ?: 'unset',
            'allowed_chat_id'  => $config['allowed_chat_id'] ?: 'pending_first_message',
            'last_update_id'   => $state['last_update_id'],
        ], JSON_PRETTY_PRINT) . "\n";
        break;

    default:
        echo "Telegram Bridge CLI Commands:\n";
        echo "  configure <TOKEN> <CHAT_ID>  - Save Telegram Bot credentials\n";
        echo "  send <MESSAGE>              - Send Markdown text message\n";
        echo "  send-interactive <MSG> [OPT]- Send message with inline action buttons\n";
        echo "  progress-update <STATUS>    - Live-update persistent progress card\n";
        echo "  wait-message [TIMEOUT]      - Long-poll for next user message/callback (0-token idle)\n";
        echo "  status                      - Check configuration status\n";
        break;
}
