//! Navidrome's Subsonic REST API (`/rest/*.view`), authenticated with
//! plaintext `u`/`p` query params and `f=json` for a JSON (not XML) body.
//! Requests ride orca's `http.request` capability (`delegated-http`).

#![allow(clippy::disallowed_types)]

use plugin_toolkit::api_client::ApiClientBuilder;
use plugin_toolkit::prelude::*;
use plugin_toolkit::progenitor_client::QueryParam;
use plugin_toolkit::reqwest::Client as HttpClient;
use plugin_toolkit::serde_json::{self, Value};

/// Subsonic protocol version this client speaks. Navidrome accepts older
/// clients too, but pinning one keeps responses predictable.
const API_VERSION: &str = "1.16.1";
const CLIENT_NAME: &str = "orca-navidrome";

pub struct Navidrome {
    base_url: String,
    username: String,
    password: String,
    http: HttpClient,
}

#[orca_struct]
#[derive(Debug, Clone, PartialEq)]
pub struct ServerInfo {
    /// Subsonic protocol version the server reports.
    pub version: String,
    /// Server implementation, e.g. "navidrome".
    pub server_type: String,
    /// Navidrome's own version string.
    pub server_version: String,
    pub open_subsonic: bool,
}

#[orca_struct]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MusicFolder {
    pub id: String,
    pub name: String,
}

#[orca_struct]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanStatus {
    pub scanning: bool,
    /// Files processed so far in the current (or last) scan.
    pub count: i64,
}

#[orca_struct]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NowPlayingEntry {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub username: String,
    pub minutes_ago: i64,
}

impl Navidrome {
    pub fn new(base_url: &str, username: &str, password: &str, insecure: bool) -> Result<Self> {
        let http = ApiClientBuilder::new().insecure(insecure).build()?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            username: username.to_string(),
            password: password.to_string(),
            http,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn url(&self, endpoint: &str) -> String {
        format!("{}/rest/{endpoint}", self.base_url)
    }

    fn base_params(&self) -> Vec<(&'static str, String)> {
        vec![
            ("u", self.username.clone()),
            ("p", self.password.clone()),
            ("v", API_VERSION.to_string()),
            ("c", CLIENT_NAME.to_string()),
            ("f", "json".to_string()),
        ]
    }

    /// `GET` a `.view` endpoint, unwrap the `subsonic-response` envelope, and
    /// return its body on `status: "ok"`.
    async fn call(&self, endpoint: &str, extra: &[(&str, String)]) -> Result<Value> {
        let mut params = self.base_params();
        params.extend(extra.iter().cloned());
        let req = params
            .iter()
            .fold(self.http.get(self.url(endpoint)), |r, (k, v)| {
                r.query(&QueryParam::new(k, v))
            });
        let resp = req.send().await.map_err(|e| anyhow!("{endpoint}: {e}"))?;
        let status = resp.status();
        let body = resp.text().await.map_err(|e| anyhow!("{endpoint}: {e}"))?;
        if !status.is_success() {
            bail!(
                "{endpoint}: HTTP {} {}",
                status.as_u16(),
                error_excerpt(&body)
            );
        }
        let v: Value =
            serde_json::from_str(&body).map_err(|e| anyhow!("{endpoint}: decode: {e}"))?;
        let root = &v["subsonic-response"];
        match root["status"].as_str() {
            Some("ok") => Ok(root.clone()),
            Some("failed") => bail!(
                "{endpoint}: subsonic error {}: {}",
                root["error"]["code"].as_i64().unwrap_or(0),
                root["error"]["message"].as_str().unwrap_or("unknown"),
            ),
            _ => bail!("{endpoint}: unexpected response: {}", error_excerpt(&body)),
        }
    }

    pub async fn server_info(&self) -> Result<ServerInfo> {
        let v = self.call("ping.view", &[]).await?;
        Ok(ServerInfo {
            version: str_of(&v["version"]),
            server_type: str_of(&v["type"]),
            server_version: str_of(&v["serverVersion"]),
            open_subsonic: v["openSubsonic"].as_bool().unwrap_or(false),
        })
    }

    pub async fn music_folders(&self) -> Result<Vec<MusicFolder>> {
        let v = self.call("getMusicFolders.view", &[]).await?;
        Ok(arr(&v["musicFolders"]["musicFolder"])
            .iter()
            .map(parse_music_folder)
            .collect())
    }

    /// Queue a library scan. `full_scan` reprocesses every file instead of
    /// only changed ones.
    pub async fn start_scan(&self, full_scan: bool) -> Result<ScanStatus> {
        let extra = [("fullScan", full_scan.to_string())];
        let v = self.call("startScan.view", &extra).await?;
        parse_scan_status(&v["scanStatus"])
    }

    pub async fn scan_status(&self) -> Result<ScanStatus> {
        let v = self.call("getScanStatus.view", &[]).await?;
        parse_scan_status(&v["scanStatus"])
    }

    pub async fn now_playing(&self) -> Result<Vec<NowPlayingEntry>> {
        let v = self.call("getNowPlaying.view", &[]).await?;
        Ok(arr(&v["nowPlaying"]["entry"])
            .iter()
            .map(parse_now_playing_entry)
            .collect())
    }
}

/// Subsonic music-folder ids are plain integers on the wire.
fn id_of(v: &Value) -> String {
    match v {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        _ => String::new(),
    }
}

fn parse_music_folder(v: &Value) -> MusicFolder {
    MusicFolder {
        id: id_of(&v["id"]),
        name: str_of(&v["name"]),
    }
}

fn parse_scan_status(v: &Value) -> Result<ScanStatus> {
    Ok(ScanStatus {
        scanning: v["scanning"].as_bool().unwrap_or(false),
        count: v["count"].as_i64().unwrap_or(0),
    })
}

fn parse_now_playing_entry(v: &Value) -> NowPlayingEntry {
    NowPlayingEntry {
        id: str_of(&v["id"]),
        title: str_of(&v["title"]),
        artist: str_of(&v["artist"]),
        username: str_of(&v["username"]),
        minutes_ago: v["minutesAgo"].as_i64().unwrap_or(0),
    }
}

fn arr(v: &Value) -> &[Value] {
    v.as_array().map(Vec::as_slice).unwrap_or(&[])
}

fn str_of(v: &Value) -> String {
    v.as_str().unwrap_or_default().to_string()
}

const ERROR_EXCERPT: usize = 256;

fn error_excerpt(body: &str) -> String {
    let body = body.trim();
    let cut = body
        .char_indices()
        .nth(ERROR_EXCERPT)
        .map_or(body, |(i, _)| &body[..i]);
    plugin_toolkit::logging::scrub(cut).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    type Call = (String, String);

    fn serve_with<F>(
        route: impl Fn(&str, &str) -> Option<(u16, String)> + 'static,
        body: F,
    ) -> Vec<Call>
    where
        F: FnOnce(),
    {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = seen.clone();
        plugin_toolkit::capsink::with_cap_sink(
            Box::new(move |cap: &str, req: &str| {
                assert_eq!(cap, "http.request");
                let req: Value = serde_json::from_str(req).unwrap();
                let url = req["url"].as_str().unwrap().to_string();
                let method = req["method"].as_str().unwrap().to_string();
                let path = url.trim_start_matches("http://nav.test").to_string();
                log.lock().unwrap().push((method.clone(), path.clone()));
                let (status, body) = route(&method, &path).unwrap_or((404, "Not Found".into()));
                Ok(json!({"status": status, "headers": [], "body": body.into_bytes()}).to_string())
            }),
            body,
        );
        let out = seen.lock().unwrap().clone();
        out
    }

    fn serve<F>(path: &'static str, body: Value, f: F) -> Vec<Call>
    where
        F: FnOnce(),
    {
        serve_with(
            move |_, p| p.starts_with(path).then(|| (200, body.to_string())),
            f,
        )
    }

    fn block<T>(f: impl std::future::Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(f)
    }

    fn nav() -> Navidrome {
        Navidrome::new("http://nav.test/", "admin", "s3cret", false).unwrap()
    }

    #[test]
    fn server_info_parses_ping_response() {
        let calls = serve(
            "/rest/ping.view",
            json!({"subsonic-response": {
                "status": "ok", "version": "1.16.1", "type": "navidrome",
                "serverVersion": "0.52.0", "openSubsonic": true
            }}),
            || {
                let info = block(nav().server_info()).unwrap();
                assert_eq!(info.server_type, "navidrome");
                assert_eq!(info.server_version, "0.52.0");
                assert!(info.open_subsonic);
            },
        );
        assert_eq!(calls.len(), 1);
        assert!(calls[0].1.contains("u=admin"), "{calls:?}");
        assert!(calls[0].1.contains("p=s3cret"), "{calls:?}");
        assert!(calls[0].1.contains("f=json"), "{calls:?}");
    }

    #[test]
    fn music_folders_parses_the_list() {
        serve(
            "/rest/getMusicFolders.view",
            json!({"subsonic-response": {"status": "ok", "version": "1.16.1",
                "musicFolders": {"musicFolder": [
                    {"id": 0, "name": "Music"},
                    {"id": 1, "name": "Podcasts"}
                ]}
            }}),
            || {
                let folders = block(nav().music_folders()).unwrap();
                assert_eq!(
                    folders,
                    vec![
                        MusicFolder {
                            id: "0".into(),
                            name: "Music".into()
                        },
                        MusicFolder {
                            id: "1".into(),
                            name: "Podcasts".into()
                        },
                    ]
                );
            },
        );
    }

    #[test]
    fn start_scan_sends_full_scan_flag_and_parses_status() {
        let calls = serve(
            "/rest/startScan.view",
            json!({"subsonic-response": {"status": "ok", "version": "1.16.1",
                "scanStatus": {"scanning": true, "count": 0}
            }}),
            || {
                let status = block(nav().start_scan(true)).unwrap();
                assert!(status.scanning);
                assert_eq!(status.count, 0);
            },
        );
        assert!(calls[0].1.contains("fullScan=true"), "{calls:?}");
    }

    #[test]
    fn scan_status_parses_in_progress_counts() {
        serve(
            "/rest/getScanStatus.view",
            json!({"subsonic-response": {"status": "ok", "version": "1.16.1",
                "scanStatus": {"scanning": true, "count": 4821}
            }}),
            || {
                let status = block(nav().scan_status()).unwrap();
                assert!(status.scanning);
                assert_eq!(status.count, 4821);
            },
        );
    }

    #[test]
    fn now_playing_parses_entries() {
        serve(
            "/rest/getNowPlaying.view",
            json!({"subsonic-response": {"status": "ok", "version": "1.16.1",
                "nowPlaying": {"entry": [{
                    "id": "tr_1", "title": "Song", "artist": "Band",
                    "username": "alice", "minutesAgo": 2
                }]}
            }}),
            || {
                let entries = block(nav().now_playing()).unwrap();
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].username, "alice");
                assert_eq!(entries[0].minutes_ago, 2);
            },
        );
    }

    #[test]
    fn now_playing_is_empty_when_nothing_is_playing() {
        serve(
            "/rest/getNowPlaying.view",
            json!({"subsonic-response": {"status": "ok", "version": "1.16.1"}}),
            || assert!(block(nav().now_playing()).unwrap().is_empty()),
        );
    }

    #[test]
    fn subsonic_failure_status_surfaces_the_message() {
        serve(
            "/rest/ping.view",
            json!({"subsonic-response": {"status": "failed", "version": "1.16.1",
                "error": {"code": 40, "message": "Wrong username or password"}
            }}),
            || {
                let e = block(nav().server_info()).unwrap_err().to_string();
                assert!(e.contains("Wrong username or password"), "{e}");
            },
        );
    }

    #[test]
    fn http_errors_surface_with_status() {
        serve_with(
            |_, _| None,
            || {
                let e = block(nav().server_info()).unwrap_err().to_string();
                assert!(e.contains("HTTP 404"), "{e}");
            },
        );
    }

    #[test]
    fn error_bodies_are_truncated_and_scrubbed() {
        serve_with(
            |_, _| {
                Some((
                    500,
                    format!(
                        r#"{{"apiKey":"supersecretvalue123","pad":"{}"}}"#,
                        "x".repeat(2000)
                    ),
                ))
            },
            || {
                let e = block(nav().server_info()).unwrap_err().to_string();
                assert!(e.len() < 400, "{}", e.len());
                assert!(!e.contains("supersecretvalue123"), "{e}");
            },
        );
    }
}
