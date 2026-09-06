//! Dynamic (subprocess) entrypoint for the navidrome plugin.
//!
//! Builds the typed `Plugin` and serves it over the orca socket. The plugin is
//! a `[[bin]]`, owns no runtime, and reaches orca only through the socket.
plugin_toolkit::instrument::bootstrap!();
use navidrome::NavidromeBackend;
use plugin_toolkit::plugin::Plugin;

fn main() -> plugin_toolkit::anyhow::Result<()> {
    Plugin::named("navidrome")
        .version(env!("CARGO_PKG_VERSION"))
        .service(NavidromeBackend::new("navidrome"))
        .serve()
}
