<?php
/**
 * Optimized Nextcloud Server Configuration Template for Windows All-In-One
 * High Performance, Redis Transactional Locking, Cloudflare Tunnel & Direct Drive Access
 */

$CONFIG = [
  'instanceid' => 'oc_auto_generated',
  'passwordsalt' => 'auto_generated_salt',
  'secret' => 'auto_generated_secret',
  'trusted_domains' => [
    0 => 'localhost:8080',
    1 => '127.0.0.1:8080',
    2 => '*.trycloudflare.com',
    3 => '*.yourdomain.com',
  ],

  // Storage and Path Configuration
  'datadirectory' => 'C:\\NextcloudAIO\\data',
  'dbtype' => 'mysql', // or 'sqlite3' for ultra-lite mode
  'version' => '30.0.2',
  'overwrite.cli.url' => 'http://localhost:8080',
  'installed' => false,

  // Cloudflare Tunnel Reverse Proxy Integration
  'overwriteprotocol' => 'https',
  'trusted_proxies' => [
    '127.0.0.1',
    '::1',
  ],
  'forwarded_for_headers' => [
    'HTTP_X_FORWARDED_FOR',
    'HTTP_CF_CONNECTING_IP',
  ],

  // High Performance Caching & File Locking
  'memcache.local' => '\OC\Memcache\APCu',
  'memcache.distributed' => '\OC\Memcache\Redis',
  'memcache.locking' => '\OC\Memcache\Redis',
  'filelocking.enabled' => true,
  'redis' => [
    'host' => '127.0.0.1',
    'port' => 6379,
    'timeout' => 1.5,
  ],

  // Background Jobs & Maintenance
  'maintenance_window_start' => 1,
  'default_phone_region' => 'US',

  // App & Direct Drive Access Capabilities
  'files_external_allow_create_new_local' => true,
  'enable_previews' => true,
  'preview_max_x' => 2048,
  'preview_max_y' => 2048,
  'preview_max_memory' => 512,
  'preview_max_filesize_image' => 50,
  'enabledPreviewProviders' => [
    'OC\Preview\PNG',
    'OC\Preview\JPEG',
    'OC\Preview\GIF',
    'OC\Preview\BMP',
    'OC\Preview\XBitmap',
    'OC\Preview\MP3',
    'OC\Preview\TXT',
    'OC\Preview\MarkDown',
    'OC\Preview\OpenDocument',
    'OC\Preview\Krita',
    'OC\Preview\HEIC',
    'OC\Preview\Movie',
  ],
];
