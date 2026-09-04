import { useEffect, useRef, useState } from "react";
import type { Capability, MetricsEnvelope, StatusResponse } from "./types";

type ConnectionState = "connecting" | "live" | "polling" | "stale" | "offline";

const FRESHNESS_MS = 5_000;
const POLL_INTERVAL_MS = 5_000;
const RECONNECT_BASE_MS = 2_000;
const RECONNECT_MAX_MS = 30_000;

function isMetricsEnvelope(value: unknown): value is MetricsEnvelope {
  if (!value || typeof value !== "object") return false;
  const obj = value as Record<string, unknown>;
  return (
    typeof obj.schema_version === "number" &&
    typeof obj.timestamp_unix_ms === "number" &&
    typeof obj.router === "object" &&
    obj.router !== null &&
    typeof obj.runtime === "object" &&
    obj.runtime !== null &&
    typeof obj.system === "object" &&
    obj.system !== null
  );
}

async function getJson<T>(path: string, signal?: AbortSignal): Promise<T> {
  const response = await fetch(path, {
    headers: { Accept: "application/json" },
    signal,
  });
  if (!response.ok) throw new Error(`${path}: HTTP ${response.status}`);
  return response.json() as Promise<T>;
}

export function useRouterData() {
  const [metrics, setMetrics] = useState<MetricsEnvelope | null>(null);
  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [capabilities, setCapabilities] = useState<Capability[]>([]);
  const [connection, setConnection] = useState<ConnectionState>("connecting");
  const lastSeq = useRef(0);
  const lastSeenAt = useRef(0);
  const pollInFlight = useRef(false);
  const reconnectAttempt = useRef(0);

  const refreshMeta = async (signal?: AbortSignal) => {
    try {
      const nextStatus = await getJson<StatusResponse>("/api/status", signal);
      setStatus(nextStatus);
    } catch {
      /* keep last known status */
    }
    try {
      const caps = await getJson<{ capabilities: Capability[] }>("/api/capabilities", signal);
      setCapabilities(caps.capabilities);
    } catch {
      /* keep last known capabilities */
    }
  };

  useEffect(() => {
    const controller = new AbortController();
    void refreshMeta(controller.signal).catch(() => setConnection("offline"));
    return () => controller.abort();
  }, []);

  useEffect(() => {
    let stopped = false;
    let polling: number | undefined;
    let reconnect: number | undefined;
    let freshness: number | undefined;
    let socket: WebSocket | undefined;

    const markFresh = (envelope: MetricsEnvelope) => {
      lastSeenAt.current = Date.now();
      if (typeof envelope.sequence === "number") {
        lastSeq.current = envelope.sequence;
      }
      setMetrics(envelope);
      setConnection("live");
    };

    const pollOnce = async () => {
      if (pollInFlight.current || stopped) return;
      pollInFlight.current = true;
      const controller = new AbortController();
      const timer = window.setTimeout(() => controller.abort(), 4_000);
      try {
        const payload = await getJson<unknown>("/api/metrics/snapshot", controller.signal);
        if (!isMetricsEnvelope(payload)) {
          setConnection("offline");
          return;
        }
        lastSeenAt.current = Date.now();
        setMetrics(payload);
        setConnection("polling");
        await refreshMeta(controller.signal);
      } catch {
        setConnection("offline");
      } finally {
        window.clearTimeout(timer);
        pollInFlight.current = false;
      }
    };

    const startPolling = () => {
      void pollOnce();
      if (polling) window.clearInterval(polling);
      polling = window.setInterval(() => void pollOnce(), POLL_INTERVAL_MS);
    };

    const connect = () => {
      if (stopped) return;
      const scheme = window.location.protocol === "https:" ? "wss" : "ws";
      socket = new WebSocket(`${scheme}://${window.location.host}/api/ws/metrics`);
      (window as unknown as { __dbrMetricsSocket?: WebSocket }).__dbrMetricsSocket = socket;
      socket.onopen = () => {
        reconnectAttempt.current = 0;
        setConnection("connecting");
      };
      socket.onmessage = (event) => {
        try {
          const parsed: unknown = JSON.parse(String(event.data));
          if (!isMetricsEnvelope(parsed)) {
            setConnection("stale");
            return;
          }
          if (
            typeof parsed.sequence === "number" &&
            lastSeq.current > 0 &&
            parsed.sequence < lastSeq.current
          ) {
            setConnection("stale");
            return;
          }
          markFresh(parsed);
        } catch {
          setConnection("stale");
        }
      };
      socket.onclose = () => {
        if (stopped) return;
        startPolling();
        const delay = Math.min(
          RECONNECT_MAX_MS,
          RECONNECT_BASE_MS * 2 ** reconnectAttempt.current,
        );
        reconnectAttempt.current += 1;
        reconnect = window.setTimeout(() => {
          if (polling) window.clearInterval(polling);
          setConnection("connecting");
          connect();
        }, delay);
      };
    };

    freshness = window.setInterval(() => {
      if (!lastSeenAt.current) return;
      if (Date.now() - lastSeenAt.current > FRESHNESS_MS) {
        setConnection((prev) => (prev === "live" ? "stale" : prev));
      }
    }, 1_000);

    connect();
    return () => {
      stopped = true;
      socket?.close();
      if (polling) window.clearInterval(polling);
      if (reconnect) window.clearTimeout(reconnect);
      if (freshness) window.clearInterval(freshness);
    };
  }, []);

  return { metrics, status, capabilities, connection };
}
