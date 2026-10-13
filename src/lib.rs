//! navidrome service backend — Music streaming (Subsonic API).
//!
//! Implements `ServiceBackend` so the generic `service.*` tools
//! (deploy/backup/restore/configure/status/connect/sync) drive navidrome. No
//! `#[orca_tool]`s — the only orca dep is `plugin-toolkit`. Modeled on the
//! nfs StorageBackend. See orca/docs/PLUGIN-PROGRAM.md.
#![allow(clippy::disallowed_types)]

use plugin_toolkit::service::{
    BoxFuture, Routes, Runtime, ServiceBackend, ServiceError, ServiceStatus, WorkloadSpec,
};

/// navidrome backend. Holds only the provider name; the instance id and its
/// `Routes` come from the generic `service.*` tools on each op.
#[derive(Debug, Clone)]
pub struct NavidromeBackend {
    provider: &'static str,
}

impl NavidromeBackend {
    pub fn new(provider: &'static str) -> Self {
        Self { provider }
    }
}

impl ServiceBackend for NavidromeBackend {
    fn provider(&self) -> &str {
        self.provider
    }

    /// Runtimes navidrome can be placed on. `service.deploy` hands the
    /// `workload_spec` below to a matching deploy target — this backend never
    /// drives pct/docker itself (that mechanic lives in the deploy-target domain).
    fn runtimes(&self) -> Vec<Runtime> {
        vec![Runtime::Docker, Runtime::Podman, Runtime::Lxc, Runtime::Vm]
    }

    fn default_port(&self) -> u16 {
        4533
    }

    /// In-workload paths holding config/data. This is ALL navidrome declares for
    /// backup — the generic pluggable backup (tar for containers/LXC, PBS for
    /// Proxmox guests when available) snapshots these. No backup/restore code
    /// here; those are inherited from ServiceBackend's defaults.
    fn data_paths(&self) -> Vec<String> {
        vec!["/config".to_string()]
    }

    fn workload_spec<'a>(
        &'a self,
        _runtime: Runtime,
        _instance: &'a str,
        _routes: &'a Routes,
    ) -> BoxFuture<'a, Result<WorkloadSpec, ServiceError>> {
        // TODO: describe the navidrome workload (image/template, ports, mounts,
        // env) for the chosen runtime. The deploy target turns this into a
        // compose service / LXC config / VM. See deploy-target::WorkloadSpec.
        Box::pin(async move { Err(ServiceError::unimplemented("navidrome.workload_spec")) })
    }

    fn configure<'a>(
        &'a self,
        _instance: &'a str,
        _routes: &'a Routes,
        _config: &'a str,
    ) -> BoxFuture<'a, Result<(), ServiceError>> {
        // TODO: apply navidrome-specific config idempotently.
        Box::pin(async move { Err(ServiceError::unimplemented("navidrome.configure")) })
    }

    fn status<'a>(
        &'a self,
        _instance: &'a str,
        _routes: &'a Routes,
    ) -> BoxFuture<'a, Result<ServiceStatus, ServiceError>> {
        // TODO: real health/diagnostics.
        Box::pin(async move { Err(ServiceError::unimplemented("navidrome.status")) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declares_provider() {
        let b = NavidromeBackend::new("navidrome");
        assert_eq!(b.provider(), "navidrome");
    }
}
