# Navidrome

Lightweight self-hosted music server. Subsonic API compatible — works with DSub, Symfonium, Feishin, and other clients.

- **Host**: <host> (<ip>)
- **Port**: 4533
- **Image**: `deluan/navidrome`
- **Compose**: [compose/navidrome/docker-compose.yml](../../compose/navidrome/docker-compose.yml)

## Volumes

| Host Path | Container Path | Description |
|-----------|---------------|-------------|
| `/opt/appdata/navidrome` | `/data` | Navidrome config and database |
| `/mnt/<host>/data/media/music` | `/music` | Music library (NFS, read-only) |

## Mobile / Desktop Clients

| Client | Platform | Protocol |
|--------|----------|----------|
| Symfonium | Android | Subsonic |
| DSub | Android | Subsonic |
| Feishin | Desktop | Subsonic/OpenSubsonic |
| Sonic Channel | iOS | Subsonic |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TZ` | `Etc/UTC` | Timezone |
| `ND_SCANSCHEDULE` | `1h` | Library scan interval |
| `ND_LOGLEVEL` | `info` | Log level |
| `ND_SESSIONTIMEOUT` | `24h` | Session timeout |
| `ND_BASEURL` | *(empty)* | Base URL if behind reverse proxy |
| `NAVIDROME_IMAGE_TAG` | `latest` | Image tag |
| `NAVIDROME_CONFIG_PATH` | `/opt/appdata/navidrome` | Config/data directory |
| `NAVIDROME_PORT` | `4533` | Host port |
| `MEDIA_PATH` | `/mnt/<host>/data/media` | Media library base path |

## Deploy

Deployed as a Portainer Git stack from `<github-org>/<repo>` main branch. Auto-updates every 5 minutes.

## Troubleshooting

```bash
docker logs navidrome
mount | grep <host>  # verify NFS mounts
```

## Scan-on-import integration (orca)

**Goal (fleet principle):** when a downloader/manager finishes a file, the server
that hosts it is *told* immediately — not left to a periodic rescan. Navidrome's
`ND_SCANSCHEDULE` (1h) and filesystem watcher are the fallback, but the watcher
does **not** fire over the NFS/SMB music share, so a push is required.

Navidrome has no native *arr connector, so the push is a **Lidarr Custom Script**
that calls Navidrome's Subsonic `startScan` endpoint on import/upgrade/rename.

### API used

Navidrome speaks the Subsonic/OpenSubsonic API. Trigger a scan with:

```
GET {NAVIDROME_URL}/rest/startScan.view?u={user}&t={token}&s={salt}&v=1.16.1&c=orca&f=json
```

- **Auth (token, preferred):** `salt` = random hex; `token` = `md5(password + salt)`.
  Avoids putting the password on the wire. (`u`+`p=` plaintext also works but is not used.)
- **Success:** `{"subsonic-response":{"status":"ok","scanStatus":{"scanning":true,...}}}`.
- **Other useful ops:** `getScanStatus.view` (progress), `ping.view` (health/auth check).
- Scan requires an **admin** Subsonic user.

### Live wiring (current, pre-plugin)

- **Lidarr** (`freyr`, docker) → connector **`navidrome-scan`** (Custom Script,
  `onReleaseImport` + `onUpgrade` + `onRename`).
- Script: `/config/scripts/navidrome-scan.sh` (in the lidarr appdata) sources
  creds from the sibling `navidrome.env` (mode `600`) and calls `startScan` with
  token auth. Handles the connector's `Test` event so the UI test button works.
- Navidrome target: `http://10.0.0.6:4533` (baldur), admin user `skey`.

### Credentials

Stored in 1Password: item **`navidrome`** (URL / admin user / password). orca-vault
copy: **`navidrome (orca)`**. The Custom Script's `navidrome.env` is the only
on-disk copy of the password (app appdata, `600`, app-uid owned).

### Future plugin capability

This belongs in the navidrome plugin's `configure` (see [CAPABILITIES.md](../CAPABILITIES.md)):
orca should (1) provision a dedicated least-privilege scan user, (2) register the
scan-on-import hook on the relevant manager(s), and (3) expose a `scan` action +
`getScanStatus` in `status`. The hand-installed Custom Script above is the interim
step and the spec for that code.
