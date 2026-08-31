export type DataPlaneState = "disabled" | "starting" | "operational" | "degraded" | "faulted";

export interface RouterMetrics {
  bip_rx_packets: number;
  bip_tx_packets: number;
  mstp_rx_packets: number;
  mstp_tx_packets: number;
  forwarded_bip_to_mstp: number;
  forwarded_mstp_to_bip: number;
  dropped_packets: number;
  invalid_frames: number;
  header_crc_errors: number;
  data_crc_errors: number;
  apdu_timeouts: number;
  serial_reconnects: number;
  tx_tokens: number;
  rx_tokens: number;
  tx_poll_for_master: number;
  rx_poll_for_master: number;
  event_count: number;
}

export interface RuntimeMetrics {
  data_plane: DataPlaneState;
  bip_link: DataPlaneState;
  mstp_link: DataPlaneState;
  rfsm_state: string;
  mnsm_state: string;
  next_station: number | null;
  poll_station: number | null;
  silence_timer_ms: number;
  last_error: string | null;
}

export interface SystemMetrics {
  uptime_seconds: number;
  load_1m: number;
  load_5m: number;
  load_15m: number;
  cpu_percent: number;
  memory_total_bytes: number;
  memory_available_bytes: number;
  process_rss_bytes: number;
  temperature_celsius: number | null;
}

export interface MetricsEnvelope {
  schema_version: number;
  timestamp_unix_ms: number;
  router: RouterMetrics;
  runtime: RuntimeMetrics;
  system: SystemMetrics;
}

export interface StatusResponse {
  name: string;
  location: string;
  version: string;
  git_sha: string;
  rusty_bacnet_rev: string;
  runtime: RuntimeMetrics;
}

export interface Capability {
  id: string;
  state: "available" | "experimental" | "not_implemented" | "blocked_by_evidence";
  detail: string;
}

