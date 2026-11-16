<?php
/**
 * ==============================================================================
 * Optimized Master Configuration for Nextcloud Server on Windows (All-In-One)
 * Tailored for High Performance, Zero-Deadlock Redis Locking, Cloudflare Tunnel,
 * Direct Drive Access, and Comprehensive Core & 3rd-Party App Support.
 * ==============================================================================
 */

$CONFIG = [
  // ----------------------------------------------------------------------------
  // 1. Instance Identification & Security Salts
  // ----------------------------------------------------------------------------
  'instanceid' => 'oc_auto_generated',
  'passwordsalt' => 'auto_generated_salt',
  'secret' => 'auto_generated_secret',
  'version' => '30.0.2',
  'installed' => false,

  // ----------------------------------------------------------------------------
  // 2. Network, Domain & Cloudflare Tunnel Proxy Configuration
  // ----------------------------------------------------------------------------
  'trusted_domains' => [
    0 => 'localhost:8080',
    1 => '127.0.0.1:8080',
    2 => 'localhost:8081',
    3 => '*.trycloudflare.com',
    4 => '*.yourdomain.com',
  ],

  // Overwrite protocol ensures HTTPS redirection and valid WebDAV links over Cloudflare
  'overwriteprotocol' => 'https',
  'overwrite.cli.url' => 'http://localhost:8080',
  'trusted_proxies' => [
    '127.0.0.1',
    '::1',
    '10.0.0.0/8',
    '172.16.0.0/12',
    '192.168.0.0/16',
  ],
  'forwarded_for_headers' => [
    'HTTP_X_FORWARDED_FOR',
    'HTTP_CF_CONNECTING_IP',
  ],

  // ----------------------------------------------------------------------------
  // 3. Storage, Paths & Direct Drive Access (files_external)
  // ----------------------------------------------------------------------------
  'datadirectory' => 'C:\\NextcloudAIO\\data',
  'skeletondirectory' => '', // Clean onboarding: disable default heavy demo files
  'files_external_allow_create_new_local' => true,
  'filesystem_check_changes' => 1, // Auto-detects files added to physical drives in Windows
  'allow_local_remote_servers' => true,
  'quota_include_external_storage' => false,

  // ----------------------------------------------------------------------------
  // 4. High-Performance Caching & Transactional Locking (Redis + APCu)
  // ----------------------------------------------------------------------------
  'memcache.local' => '\OC\Memcache\APCu',
  'memcache.distributed' => '\OC\Memcache\Redis',
  'memcache.locking' => '\OC\Memcache\Redis',
  'filelocking.enabled' => true,
  'filelocking.ttl' => 3600,
  'redis' => [
    'host' => '127.0.0.1',
    'port' => 6379,
    'timeout' => 1.5,
    'read_timeout' => 1.5,
    'dbindex' => 0,
  ],

  // ----------------------------------------------------------------------------
  // 5. Database Connection
  // ----------------------------------------------------------------------------
  // Production default: MariaDB / MySQL (or 'sqlite3' for ultra-lite mode)
  'dbtype' => 'mysql',
  'dbname' => 'nextcloud',
  'dbhost' => '127.0.0.1:3306',
  'dbport' => '',
  'dbtableprefix' => 'oc_',
  'dbuser' => 'nextcloud',
  'dbpassword' => 'nextcloud_secure_password',
  'mysql.utf8mb4' => true,

  // ----------------------------------------------------------------------------
  // 6. Media & Image Previews (Memories, Photos, Video Streaming)
  // ----------------------------------------------------------------------------
  'enable_previews' => true,
  'preview_max_x' => 2048,
  'preview_max_y' => 2048,
  'preview_max_memory' => 512,
  'preview_max_filesize_image' => 50, // Max 50MB image for preview generation
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
    'OC\Preview\TIFF',
    'OC\Preview\Movie',
    'OC\Preview\PDF',
    'OC\Preview\SVG',
    'OC\Preview\Photoshop',
  ],

  // ----------------------------------------------------------------------------
  // 7. Background Jobs, Maintenance & Performance Scheduling
  // ----------------------------------------------------------------------------
  'maintenance_window_start' => 1, // Runs background jobs between 01:00 and 05:00 UTC
  'default_phone_region' => 'US',
  'activity_expire_days' => 30,
  'trashbin_retention_obligation' => 'auto, 30',
  'versions_retention_obligation' => 'auto, 30',

  // ----------------------------------------------------------------------------
  // 8. Logging & Diagnostics
  // ----------------------------------------------------------------------------
  'log_type' => 'file',
  'logfile' => 'logs/nextcloud.log',
  'loglevel' => 2, // 0 = DEBUG, 1 = INFO, 2 = WARN, 3 = ERROR
  'logdateformat' => 'Y-m-d H:i:s',
  'log_rotate_size' => 104857600, // 100 MB max log file size
  'log_max_files' => 3,

  // ----------------------------------------------------------------------------
  // 9. Third-Party App Specific Settings
  // ----------------------------------------------------------------------------
  // Nextcloud Office (Collabora Online)
  'collabora_wopi_url' => 'http://127.0.0.1:9980',

  // Nextcloud Talk (Spreed WebRTC)
  'spreed_has_signaling' => false,

  // Large File Upload Chunking (20MB chunks)
  'chunk_size' => 20971520,
];
