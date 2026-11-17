<?php
/**
 * Nextcloud Headless Auto-Configuration Template
 * Placed in 'nextcloud/config/autoconfig.php' for instant zero-touch setup
 */

$AUTOCONFIG = [
  'dbtype'        => 'sqlite', // Default zero-config database (or 'mysql' if MariaDB selected)
  'dbname'        => 'nextcloud',
  'dbuser'        => 'nextcloud',
  'dbpass'        => 'nextcloud_secure_pass',
  'dbhost'        => '127.0.0.1:3306',
  'dbtableprefix' => 'oc_',
  'adminlogin'    => 'superadmin',
  'adminpass'     => 'SuperNextcloudPass2026!',
  'directory'     => 'C:\\NextcloudAIO\\data',
];
