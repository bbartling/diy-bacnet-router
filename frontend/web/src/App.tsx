import { useState } from "react";
import { useRouterData } from "./useRouterData";
import type { Capability, MetricsEnvelope } from "./types";

type Page = "overview" | "mstp" | "ip" | "system" | "configuration";

const number = new Intl.NumberFormat("en-US");

function formatBytes(value: number): string {
  if (!value) return "—";
  const units = ["B", "KB", "MB", "GB"];
  let current = value;
  let index = 0;
  while (current >= 1024 && index < units.length - 1) {
    current /= 1024;
    index += 1;
  }
  return `${current.toFixed(index > 1 ? 1 : 0)} ${units[index]}`;
}

function Stat({ label, value, detail }: { label: string; value: string; detail?: string }) {
  return (
    <article className="stat-card">
      <span>{label}</span>
      <strong>{value}</strong>
      {detail && <small>{detail}</small>}
    </article>
  );
}

function Gate({ capability }: { capability: Capability }) {
  return (
    <li>
      <span className={`gate gate-${capability.state}`}>{capability.state.replaceAll("_", " ")}</span>
      <div><strong>{capability.id.replaceAll("_", " ")}</strong><small>{capability.detail}</small></div>
    </li>
  );
}

function Overview({ data, capabilities }: { data: MetricsEnvelope | null; capabilities: Capability[] }) {
  const m = data?.router;
  const system = data?.system;
  const runtime = data?.runtime;
  return (
    <>
      <section className="hero-grid">
        <div className="route-map panel">
          <div className="port"><span>BACnet/IP</span><strong>{runtime?.bip_link ?? "disabled"}</strong></div>
          <div className="flow"><i /> <span>NPDU router</span> <i /></div>
          <div className="port"><span>MS/TP</span><strong>{runtime?.mstp_link ?? "disabled"}</strong></div>
        </div>
        <div className="panel warning-panel">
          <span className="eyebrow">Commissioning state</span>
          <h2>Forwarding is locked</h2>
          <p>The management plane is running. Hardware routing remains fail-closed until the isolated forwarding gates pass.</p>
          <p><small>Last error: {runtime?.last_error ?? "none"}</small></p>
        </div>
      </section>
      <section className="stats-grid">
        <Stat label="B/IP received" value={number.format(m?.bip_rx_packets ?? 0)} />
        <Stat label="MS/TP received" value={number.format(m?.mstp_rx_packets ?? 0)} />
        <Stat label="Tokens received" value={number.format(m?.rx_tokens ?? 0)} />
        <Stat label="Invalid frames" value={number.format(m?.invalid_frames ?? 0)} />
        <Stat label="Event count" value={number.format(m?.event_count ?? 0)} />
        <Stat label="Serial reconnects" value={number.format(m?.serial_reconnects ?? 0)} />
        <Stat label="CPU" value={`${(system?.cpu_percent ?? 0).toFixed(1)}%`} detail={`Load ${system?.load_1m.toFixed(2) ?? "—"}`} />
        <Stat label="Available memory" value={formatBytes(system?.memory_available_bytes ?? 0)} detail={`Process ${formatBytes(system?.process_rss_bytes ?? 0)}`} />
      </section>
      <section className="panel">
        <div className="section-heading"><div><span className="eyebrow">Evidence ledger</span><h2>Capabilities</h2></div></div>
        <ul className="gate-list">{capabilities.map((item) => <Gate key={item.id} capability={item} />)}</ul>
      </section>
    </>
  );
}

function Mstp({ data }: { data: MetricsEnvelope | null }) {
  const m = data?.router;
  return <section className="stats-grid">
    <Stat label="Incoming packets" value={number.format(m?.mstp_rx_packets ?? 0)} />
    <Stat label="Outgoing packets" value={number.format(m?.mstp_tx_packets ?? 0)} />
    <Stat label="TX token" value={number.format(m?.tx_tokens ?? 0)} />
    <Stat label="RX token" value={number.format(m?.rx_tokens ?? 0)} />
    <Stat label="TX Poll For Master" value={number.format(m?.tx_poll_for_master ?? 0)} />
    <Stat label="RX Poll For Master" value={number.format(m?.rx_poll_for_master ?? 0)} />
    <Stat label="Header CRC errors" value={number.format(m?.header_crc_errors ?? 0)} />
    <Stat label="Data CRC errors" value={number.format(m?.data_crc_errors ?? 0)} />
    <Stat label="Silence timer" value={`${data?.runtime.silence_timer_ms ?? 0} ms`} />
    <Stat label="Next station" value={String(data?.runtime.next_station ?? "—")} />
    <Stat label="Poll station" value={String(data?.runtime.poll_station ?? "—")} />
    <Stat label="RFSM" value={data?.runtime.rfsm_state ?? "not started"} />
    <Stat label="MNSM" value={data?.runtime.mnsm_state ?? "not started"} />
    <Stat label="Serial reconnects" value={number.format(m?.serial_reconnects ?? 0)} />
    <Stat label="Last error" value={data?.runtime.last_error ?? "none"} />
  </section>;
}

function Ip({ data }: { data: MetricsEnvelope | null }) {
  const m = data?.router;
  return <section className="stats-grid">
    <Stat label="Incoming packets" value={number.format(m?.bip_rx_packets ?? 0)} />
    <Stat label="Outgoing packets" value={number.format(m?.bip_tx_packets ?? 0)} />
    <Stat label="B/IP → MS/TP" value={number.format(m?.forwarded_bip_to_mstp ?? 0)} />
    <Stat label="MS/TP → B/IP" value={number.format(m?.forwarded_mstp_to_bip ?? 0)} />
    <Stat label="Dropped packets" value={number.format(m?.dropped_packets ?? 0)} />
    <Stat label="APDU timeouts" value={number.format(m?.apdu_timeouts ?? 0)} />
  </section>;
}

function System({ data }: { data: MetricsEnvelope | null }) {
  const s = data?.system;
  const memoryUsed = s && s.memory_total_bytes > 0 ? 100 * (1 - s.memory_available_bytes / s.memory_total_bytes) : 0;
  return <section className="stats-grid">
    <Stat label="CPU utilization" value={`${(s?.cpu_percent ?? 0).toFixed(1)}%`} />
    <Stat label="Memory used" value={`${memoryUsed.toFixed(1)}%`} detail={`${formatBytes(s?.memory_total_bytes ?? 0)} total`} />
    <Stat label="Process RSS" value={formatBytes(s?.process_rss_bytes ?? 0)} />
    <Stat label="Load average" value={(s?.load_1m ?? 0).toFixed(2)} detail={`${s?.load_5m.toFixed(2) ?? "—"} / ${s?.load_15m.toFixed(2) ?? "—"}`} />
    <Stat label="Temperature" value={s?.temperature_celsius == null ? "—" : `${s.temperature_celsius.toFixed(1)} °C`} />
    <Stat label="Host uptime" value={`${Math.floor((s?.uptime_seconds ?? 0) / 3600)} h`} />
    <Stat label="Event count" value={number.format(data?.router.event_count ?? 0)} />
    <Stat label="Last error" value={data?.runtime.last_error ?? "none"} />
  </section>;
}

function Configuration() {
  return <section className="panel config-panel">
    <span className="eyebrow">Read-only milestone</span>
    <h2>Configuration is managed over SSH</h2>
    <p>Edit <code>/etc/diy-bacnet-router/router.toml</code>, validate it with <code>--check-config</code> (no socket bind), then restart the service. Browser writes stay disabled until authentication, audit logging, atomic persistence and rollback gates pass.</p>
    <div className="command">sudo vi /etc/diy-bacnet-router/router.toml<br />sudo diy-bacnet-router --check-config --config /etc/diy-bacnet-router/router.toml<br />sudo service diy-bacnet-router restart</div>
    <a className="button-link" href="/api/config/effective">View effective JSON</a>
  </section>;
}

export function App() {
  const [page, setPage] = useState<Page>("overview");
  const { metrics, status, capabilities, connection } = useRouterData();
  const pages: Page[] = ["overview", "mstp", "ip", "system", "configuration"];
  return (
    <div className="shell">
      <header>
        <div className="brand-mark"><span>DBR</span></div>
        <div className="brand">
          <strong>DIY BACnet Router</strong>
          <span>{status?.name ?? "appliance scaffold"} · v{status?.version ?? "…"}</span>
        </div>
        <div className={`connection connection-${connection}`}><i />{connection}</div>
      </header>
      <aside>
        <div className="device-summary"><span className="eyebrow">Data plane</span><strong>{metrics?.runtime.data_plane ?? "disabled"}</strong><small>{status?.location ?? "test bench"}</small></div>
        <nav>{pages.map((item) => <button key={item} className={page === item ? "active" : ""} onClick={() => setPage(item)}>{item === "mstp" ? "MS/TP" : item === "ip" ? "BACnet/IP" : item}</button>)}</nav>
        <div className="build"><span>Release v{status?.version ?? "…"}</span><span>Git {status?.git_sha ?? "development"}</span><span>Stack {status?.rusty_bacnet_rev ?? "not integrated"}</span></div>
      </aside>
      <main>
        <div className="page-heading"><div><span className="eyebrow">Router management</span><h1>{page === "mstp" ? "MS/TP statistics" : page === "ip" ? "BACnet/IP statistics" : page}</h1></div><span className="timestamp">{metrics ? new Date(metrics.timestamp_unix_ms).toLocaleTimeString() : "waiting for metrics"}</span></div>
        {page === "overview" && <Overview data={metrics} capabilities={capabilities} />}
        {page === "mstp" && <Mstp data={metrics} />}
        {page === "ip" && <Ip data={metrics} />}
        {page === "system" && <System data={metrics} />}
        {page === "configuration" && <Configuration />}
      </main>
    </div>
  );
}

