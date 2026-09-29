# S-230-T6d.2a — WordPress preservation before host repurpose

Date: 2026-09-29
Branch: `main`
Status: ACTIVE — READY FOR SSH EXECUTION
Cloud mutation: **NONE**

## Goal

Preserve a locally verifiable, restorable WordPress package before the existing
Droplet is resized or repurposed for DubBridge.

## Canonical command

```bash
make do-preserve-wordpress
```

Defaults:

- host: `46.101.217.151`
- SSH user: `root`
- port: `22`
- local evidence root: `/tmp/dubbridge-t6d2a-wordpress-backup`

Overrides:

```bash
export DO_WORDPRESS_SSH_USER=<user>
export DO_WORDPRESS_SSH_HOST=<host>
export DO_WORDPRESS_SSH_PORT=<port>
```

## Preservation contract

The script:

1. verifies non-interactive SSH access;
2. locates `wp-config.php` under standard WordPress roots;
3. exports the DB with WP-CLI;
4. archives the complete WordPress directory;
5. records host/service/container inventory;
6. writes SHA-256 checksums and a manifest;
7. downloads the package to the operator machine;
8. verifies DB/archive presence, non-zero size and checksums locally.

Expected markers:

```text
REMOTE_BACKUP=PASS
T6D2A_DB_EXPORT=PASS
T6D2A_FILES_ARCHIVE=PASS
T6D2A_CHECKSUMS=PASS
T6D2A_RESTORE_MANIFEST=<path>
T6D2A_BACKUP_DIR=<path>
T6D2A_BACKUP=PASS
```

If WP-CLI is unavailable, the script blocks with `reason=wp-cli-missing` rather
than installing software or reading DB credentials automatically.

T6d.2a does not stop WordPress, modify packages, resize the Droplet, change DNS
or alter DigitalOcean resources. Snapshot creation remains optional and is not
part of the mandatory backup gate.
