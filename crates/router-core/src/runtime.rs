use std::sync::RwLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataPlaneState {
    Disabled,
    Starting,
    Operational,
    Degraded,
    Faulted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    Available,
    Experimental,
    NotImplemented,
    BlockedByEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability {
    pub id: String,
    pub state: CapabilityState,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeSnapshot {
    pub data_plane: DataPlaneState,
    pub bip_link: DataPlaneState,
    pub mstp_link: DataPlaneState,
    pub rfsm_state: String,
    pub mnsm_state: String,
    pub next_station: Option<u8>,
    pub poll_station: Option<u8>,
    pub silence_timer_ms: u64,
    pub last_error: Option<String>,
}

impl Default for RuntimeSnapshot {
    fn default() -> Self {
        Self {
            data_plane: DataPlaneState::Disabled,
            bip_link: DataPlaneState::Disabled,
            mstp_link: DataPlaneState::Disabled,
            rfsm_state: "not_started".into(),
            mnsm_state: "not_started".into(),
            next_station: None,
            poll_station: None,
            silence_timer_ms: 0,
            last_error: Some("rusty-bacnet router adapter is not integrated".into()),
        }
    }
}

#[derive(Debug, Default)]
pub struct RuntimeState {
    inner: RwLock<RuntimeSnapshot>,
}

impl RuntimeState {
    #[must_use]
    pub fn snapshot(&self) -> RuntimeSnapshot {
        self.inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn replace(&self, snapshot: RuntimeSnapshot) {
        *self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = snapshot;
    }

    #[must_use]
    pub fn capabilities() -> Vec<Capability> {
        vec![
            Capability {
                id: "management_api".into(),
                state: CapabilityState::Available,
                detail: "REST, OpenAPI, Prometheus and bounded WebSocket snapshots".into(),
            },
            Capability {
                id: "standard_mstp_frames".into(),
                state: CapabilityState::Experimental,
                detail: "proven in Vibe13; must be revalidated in this appliance".into(),
            },
            Capability {
                id: "bip_mstp_routing".into(),
                state: CapabilityState::BlockedByEvidence,
                detail: "adapter and isolated NPDU forwarding gates are open".into(),
            },
            Capability {
                id: "extended_mstp_frames".into(),
                state: CapabilityState::NotImplemented,
                detail: "no production claim until upstream implementation and tests exist".into(),
            },
            Capability {
                id: "bbmd_fdr".into(),
                state: CapabilityState::NotImplemented,
                detail: "out of initial routing milestone".into(),
            },
        ]
    }
}
