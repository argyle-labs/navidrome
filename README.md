<p align="center">
  <img src="assets/icon-256.png" width="120" alt="navidrome" />
</p>

# navidrome

Navidrome is a self-hosted music streaming server compatible with the Subsonic API.

A first-party [orca](https://github.com/argyle-labs/orca) plugin (service-backend).

This repo is **self-contained** — the steps below run navidrome **by hand, without orca**. orca automates exactly this (same image, ports, and data) through one generic surface.

---

## Run it without orca

### Docker Compose

```yaml
# compose.yml
services:
  navidrome:
    image: deluan/navidrome:latest
    container_name: navidrome
    restart: unless-stopped
    ports:
      - "4533:4533/tcp"   # web UI
    volumes:
      - ./data:/data
      - /path/to/music:/music:ro
```

```sh
docker compose up -d
```

### Other runtimes

**Podman** — the compose above works with `podman compose up -d`, or run it directly:

```sh
podman run -d --name navidrome --restart unless-stopped \
    -p 4533:4533/tcp \
    -v ./data:/data \
    -v /path/to/music:/music:ro \
    deluan/navidrome:latest
```

**LXC** — on a container-capable LXC (e.g. a Proxmox LXC with nesting enabled) run the same image via Docker/Podman as above, or install navidrome from upstream directly on the guest: <https://www.navidrome.org/>.

**VM** — install navidrome from upstream (<https://www.navidrome.org/>) or run the same container image inside the VM; expose port `4533`.

**Unraid** — add via *Community Applications*, or *Docker → Add Container* with image `deluan/navidrome:latest`, port `4533`, and the volume paths above.

### Dependencies

Requires a folder of music to mount read-only.

### Ports & data

| | |
|---|---|
| Default port | `4533` |
| Upstream | <https://www.navidrome.org/> |
| Operator notes | [navidrome.md](docs/navidrome.md) |


### Backup & restore

Back up the config/data volume(s) above — that's the whole service state (stop the container first for a clean copy). Restore by putting them back and starting it.

> With orca this is **`service.backup` / `service.restore`** — location-agnostic (docker / podman / lxc / vm), one command regardless of where navidrome runs. No per-service backup script.

## With orca

orca drives this plugin through the single generic `service.*` surface — no per-plugin tools:

```sh
orca service.deploy navidrome      # render + launch on any supported runtime
orca service.status navidrome      # health + rich diagnostics (typed payload)
orca service.backup navidrome      # location-agnostic backup (tar; PBS on Proxmox)
orca service.configure navidrome   # apply config via the upstream API
```

## Layout

- `src/` — the plugin (pure Rust): the `ServiceBackend` descriptor + `configure` / `status`.
- `docs/` — standalone operator notes.
- [CAPABILITIES.md](CAPABILITIES.md) — the service-backend contract checklist.
- `assets/` — plugin icon.
