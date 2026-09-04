import { expect, test, type Page } from "@playwright/test";

const pages = [
  { button: "overview", heading: /overview/i },
  { button: "MS/TP", heading: /MS\/TP/i },
  { button: "BACnet/IP", heading: /BACnet\/IP/i },
  { button: "system", heading: /system/i },
  { button: "configuration", heading: /configuration/i },
] as const;

async function assertNoHorizontalOverflow(page: Page) {
  const overflow = await page.evaluate(() => {
    const doc = document.documentElement;
    return doc.scrollWidth > doc.clientWidth + 1;
  });
  expect(overflow).toBe(false);
}

test.describe("management dashboard", () => {
  test("health and API contracts match the TypeScript surface", async ({ request }) => {
    const health = await request.get("/healthz");
    expect(health.ok()).toBeTruthy();
    const healthBody = await health.json();
    expect(healthBody.status).toBe("ok");
    expect(healthBody.data_plane).toBe("disabled");
    expect(healthBody.ready_to_route).toBe(false);

    const status = await request.get("/api/status");
    expect(status.ok()).toBeTruthy();
    const statusBody = await status.json();
    expect(statusBody.version).toBeTruthy();
    expect(statusBody.git_sha).toBeTruthy();
    expect(statusBody.runtime.data_plane).toBe("disabled");

    const metrics = await request.get("/api/metrics/snapshot");
    expect(metrics.ok()).toBeTruthy();
    const metricsBody = await metrics.json();
    expect(metricsBody.schema_version).toBe(1);
    expect(metricsBody.bacnet_telemetry_available).toBe(false);
    expect(typeof metricsBody.sequence).toBe("number");
    expect(metricsBody.router).toMatchObject({
      event_count: expect.any(Number),
      serial_reconnects: expect.any(Number),
    });
    expect(metricsBody.runtime.last_error === null || typeof metricsBody.runtime.last_error === "string").toBe(
      true,
    );

    const openapi = await request.get("/api/openapi.json");
    expect(openapi.ok()).toBeTruthy();
    const openapiBody = await openapi.json();
    for (const path of [
      "/healthz",
      "/api/status",
      "/api/capabilities",
      "/api/config/effective",
      "/api/metrics/snapshot",
      "/api/ws/metrics",
      "/metrics",
    ]) {
      expect(openapiBody.paths[path]).toBeTruthy();
    }

    const effective = await request.get("/api/config/effective");
    expect(effective.ok()).toBeTruthy();
    const configBody = await effective.json();
    expect(configBody.management.max_ws_connections).toBeGreaterThan(0);
  });

  test("renders all pages without overflow and shows version", async ({ page }, testInfo) => {
    const consoleErrors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") consoleErrors.push(msg.text());
    });

    await page.goto("/");
    await expect(page.getByText(/DIY BACnet Router/i).first()).toBeVisible();
    await expect(page.locator(".brand span")).toContainText(/v\d/);
    await expect(page.locator(".build")).toContainText(/Release v/);
    await expect(page.locator(".build")).toContainText(/Git /);
    await expect(page.locator(".device-summary strong")).toHaveText(/disabled/i);
    await expect(page.getByRole("heading", { name: /Forwarding is locked/i })).toBeVisible();
    await expect(page.getByText(/Event count/i).first()).toBeVisible();

    for (const item of pages) {
      await page.getByRole("button", { name: new RegExp(`^${item.button}$`, "i") }).click();
      await expect(page.locator("h1")).toHaveText(item.heading);
      await assertNoHorizontalOverflow(page);
    }

    await page.screenshot({
      path: `test-results/screenshots/${testInfo.project.name}-configuration.png`,
      fullPage: true,
    });

    expect(consoleErrors).toEqual([]);
  });

  test("keyboard navigation reaches nav controls", async ({ page }) => {
    await page.goto("/");
    await page.locator("nav button").first().focus();
    await expect(page.locator("nav button").first()).toBeFocused();
    await page.keyboard.press("Tab");
    await expect(page.locator("nav button").nth(1)).toBeFocused();
  });

  test("live WebSocket indicator and REST fallback after WS failure", async ({ page }) => {
    const consoleErrors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") consoleErrors.push(msg.text());
    });

    await page.goto("/");
    await expect
      .poll(async () => page.locator(".connection").innerText(), { timeout: 20_000 })
      .toMatch(/live/i);

    await page.evaluate(() => {
      const socket = (window as unknown as { __dbrMetricsSocket?: WebSocket }).__dbrMetricsSocket;
      socket?.close();
    });
    await expect
      .poll(async () => page.locator(".connection").innerText(), { timeout: 10_000 })
      .toMatch(/polling|offline|connecting/i);

    const metrics = await page.request.get("/api/metrics/snapshot");
    expect(metrics.ok()).toBeTruthy();

    const unexpected = consoleErrors.filter(
      (text) => !/websocket|web socket|ws|failed to fetch|net::ERR/i.test(text),
    );
    expect(unexpected).toEqual([]);
  });

  test("marks connection stale when metrics stream stalls", async ({ page }) => {
    await page.goto("/");
    await expect
      .poll(async () => page.locator(".connection").innerText(), { timeout: 20_000 })
      .toMatch(/live/i);

    await page.evaluate(() => {
      const socket = (window as unknown as { __dbrMetricsSocket?: WebSocket }).__dbrMetricsSocket;
      if (!socket) throw new Error("metrics socket missing");
      // Keep the socket open but drop updates so the freshness watchdog trips.
      socket.onmessage = () => undefined;
    });

    await expect
      .poll(async () => page.locator(".connection").innerText(), { timeout: 12_000 })
      .toMatch(/stale/i);
    await expect(page.locator(".connection-stale")).toBeVisible();
  });

  test("malformed WebSocket payload marks stale without crashing", async ({ page }) => {
    const consoleErrors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") consoleErrors.push(msg.text());
    });

    await page.goto("/");
    await expect
      .poll(async () => page.locator(".connection").innerText(), { timeout: 20_000 })
      .toMatch(/live/i);
    await expect(page.getByText(/unavailable \(scaffold zeros\)/i)).toBeVisible();

    await page.evaluate(() => {
      const socket = (window as unknown as { __dbrMetricsSocket?: WebSocket }).__dbrMetricsSocket;
      if (!socket?.onmessage) throw new Error("metrics socket missing");
      const deliver = socket.onmessage.bind(socket);
      socket.onmessage = () => undefined;
      (window as unknown as { __dbrDeliverMetrics?: (data: string) => void }).__dbrDeliverMetrics = (
        data,
      ) => deliver({ data } as MessageEvent);
      deliver({ data: "{not-json" } as MessageEvent);
    });

    await expect
      .poll(async () => page.locator(".connection").innerText(), { timeout: 5_000 })
      .toMatch(/stale/i);

    await page.evaluate(() => {
      const deliver = (window as unknown as { __dbrDeliverMetrics?: (data: string) => void })
        .__dbrDeliverMetrics;
      if (!deliver) throw new Error("metrics injector missing");
      deliver(
        JSON.stringify({
          schema_version: 1,
          timestamp_unix_ms: Date.now(),
          sequence: 9_999_999,
          bacnet_telemetry_available: false,
          router: {},
          runtime: { data_plane: "disabled" },
          system: {},
        }),
      );
    });

    await expect
      .poll(async () => page.locator(".connection").innerText(), { timeout: 5_000 })
      .toMatch(/live/i);

    const unexpected = consoleErrors.filter(
      (text) => !/websocket|web socket|ws|failed to fetch|net::ERR/i.test(text),
    );
    expect(unexpected).toEqual([]);
  });

  test("sequence rewind marks stale then recovers on fresh sequence", async ({ page }) => {
    await page.goto("/");
    await expect
      .poll(async () => page.locator(".connection").innerText(), { timeout: 20_000 })
      .toMatch(/live/i);

    await page.evaluate(() => {
      const socket = (window as unknown as { __dbrMetricsSocket?: WebSocket }).__dbrMetricsSocket;
      if (!socket?.onmessage) throw new Error("metrics socket missing");
      const deliver = socket.onmessage.bind(socket);
      socket.onmessage = () => undefined;
      (window as unknown as { __dbrDeliverMetrics?: (data: string) => void }).__dbrDeliverMetrics = (
        data,
      ) => deliver({ data } as MessageEvent);
      const envelope = (sequence: number) =>
        JSON.stringify({
          schema_version: 1,
          timestamp_unix_ms: Date.now(),
          sequence,
          bacnet_telemetry_available: false,
          router: {},
          runtime: { data_plane: "disabled" },
          system: {},
        });
      deliver({ data: envelope(100) } as MessageEvent);
      deliver({ data: envelope(50) } as MessageEvent);
    });

    await expect
      .poll(async () => page.locator(".connection").innerText(), { timeout: 5_000 })
      .toMatch(/stale/i);

    await page.evaluate(() => {
      const deliver = (window as unknown as { __dbrDeliverMetrics?: (data: string) => void })
        .__dbrDeliverMetrics;
      if (!deliver) throw new Error("metrics injector missing");
      deliver(
        JSON.stringify({
          schema_version: 1,
          timestamp_unix_ms: Date.now(),
          sequence: 10_000_000,
          bacnet_telemetry_available: false,
          router: {},
          runtime: { data_plane: "disabled" },
          system: {},
        }),
      );
    });

    await expect
      .poll(async () => page.locator(".connection").innerText(), { timeout: 5_000 })
      .toMatch(/live/i);
  });

  test("backend status refresh after WS drop uses latest /api/status", async ({ page }) => {
    let statusHits = 0;
    await page.route("**/api/status", async (route) => {
      statusHits += 1;
      if (statusHits <= 1) {
        await route.continue();
        return;
      }
      const response = await route.fetch();
      const body = await response.json();
      body.git_sha = "deadbeefcafebabe0123456789abcdef01234567";
      body.version = body.version ?? "0.0.0-test";
      await route.fulfill({
        status: response.status(),
        headers: { ...response.headers(), "content-type": "application/json" },
        body: JSON.stringify(body),
      });
    });

    await page.goto("/");
    await expect
      .poll(async () => page.locator(".connection").innerText(), { timeout: 20_000 })
      .toMatch(/live/i);

    await page.evaluate(() => {
      const socket = (window as unknown as { __dbrMetricsSocket?: WebSocket }).__dbrMetricsSocket;
      socket?.close();
    });

    await expect
      .poll(async () => page.locator(".build").innerText(), { timeout: 15_000 })
      .toMatch(/deadbeefcafebabe/i);
  });
});
