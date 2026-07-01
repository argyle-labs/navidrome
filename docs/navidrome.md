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
