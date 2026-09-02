import { useEffect, useState } from "react";
import type { Capability, MetricsEnvelope, StatusResponse } from "./types";

type ConnectionState = "connecting" | "live" | "polling" | "offline";

async function getJson<T>(path: string): Promise<T> {
  const response = await fetch(path, { headers: { Accept: "application/json" } });
  if (!response.ok) throw new Error(`${path}: HTTP ${response.status}`);
  return response.json() as Promise<T>;
}

export function useRouterData() {
  const [metrics, setMetrics] = useState<MetricsEnvelope | null>(null);
  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [capabilities, setCapabilities] = useState<Capability[]>([]);
  const [connection, setConnection] = useState<ConnectionState>("connecting");

  useEffect(() => {
    void getJson<StatusResponse>("/api/status").then(setStatus).catch(() => setConnection("offline"));
    void getJson<{ capabilities: Capability[] }>("/api/capabilities")
      .then((value) => setCapabilities(value.capabilities))
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    let stopped = false;
    let polling: number | undefined;
    let reconnect: number | undefined;
    let socket: WebSocket | undefined;

    const poll = async () => {
      try {
        setMetrics(await getJson<MetricsEnvelope>("/api/metrics/snapshot"));
        setConnection("polling");
      } catch {
        setConnection("offline");
      }
    };

    const connect = () => {
      if (stopped) return;
      const scheme = window.location.protocol === "https:" ? "wss" : "ws";
      socket = new WebSocket(`${scheme}://${window.location.host}/api/ws/metrics`);
      socket.onopen = () => setConnection("live");
      socket.onmessage = (event) => {
        try {
          setMetrics(JSON.parse(String(event.data)) as MetricsEnvelope);
          setConnection("live");
        } catch {
          setConnection("offline");
        }
      };
      socket.onclose = () => {
        if (stopped) return;
        void poll();
        polling = window.setInterval(() => void poll(), 5_000);
        reconnect = window.setTimeout(() => {
          if (polling) window.clearInterval(polling);
          setConnection("connecting");
          connect();
        }, 10_000);
      };
    };

    connect();
    return () => {
      stopped = true;
      socket?.close();
      if (polling) window.clearInterval(polling);
      if (reconnect) window.clearTimeout(reconnect);
    };
  }, []);

  return { metrics, status, capabilities, connection };
}

