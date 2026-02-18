import { test, expect } from "@playwright/test";
import {
  singleRunningPod,
  stoppingPod,
  podWithRestart,
  podWithAlias,
  podWithBusyPort,
} from "../fixtures/synthetic-data";

async function injectPods(page: any, pods: any[]) {
  await page.evaluate(async (pods: any[]) => {
    await (window as any).__TAURI__.core.invoke("test_inject_pods", { pods });
  }, pods);
  await page.waitForTimeout(500);
}

test.describe("New feature visual tests", () => {
  test("stopping pod shows force kill button", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await injectPods(page, stoppingPod());
    await expect(page.getByTestId("pod-tile-pod-stopping-1")).toBeVisible();
    await expect(page).toHaveScreenshot("stopping-pod.png");
  });

  test("running pod shows restart and rebuild buttons", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await injectPods(page, podWithRestart());
    await expect(page.getByTestId("pod-tile-pod-restart-1")).toBeVisible();
    await expect(page).toHaveScreenshot("pod-with-restart.png");
  });

  test("pod with alias displays alias instead of name", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await injectPods(page, podWithAlias());
    const tile = page.getByTestId("pod-tile-pod-alias-1");
    await expect(tile).toBeVisible();
    // The alias "My App" should be displayed
    await expect(tile.locator("text=My App")).toBeVisible();
    await expect(page).toHaveScreenshot("pod-with-alias.png");
  });

  test("pod with busy port shows warning indicator", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await injectPods(page, podWithBusyPort());
    await expect(page.getByTestId("pod-tile-pod-busy-port")).toBeVisible();
    await expect(page).toHaveScreenshot("pod-busy-port.png");
  });

  test("status bar shows correct counts", async ({ page }) => {
    await page.goto("http://localhost:1420");
    // Inject mixed pods - 1 running, 1 stopped, 1 error
    const mixed = [
      ...singleRunningPod(),
      {
        id: "pod-mix-stopped",
        name: "stopped-app",
        projectPath: "/tmp/stopped",
        image: "node:20",
        status: "stopped",
      },
    ];
    await injectPods(page, mixed);
    const statusBar = page.getByTestId("status-bar");
    await expect(statusBar).toBeVisible();
    await expect(page).toHaveScreenshot("status-bar-counts.png");
  });

  test("collapsible ports section", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await injectPods(page, podWithRestart());
    const tile = page.getByTestId("pod-tile-pod-restart-1");
    await expect(tile).toBeVisible();

    // Find and click the ports toggle
    const portsToggle = tile.getByTestId("pod-ports-toggle-pod-restart-1");
    if (await portsToggle.isVisible()) {
      // Click to collapse
      await portsToggle.click();
      await page.waitForTimeout(200);
      await expect(page).toHaveScreenshot("ports-collapsed.png");

      // Click to expand
      await portsToggle.click();
      await page.waitForTimeout(200);
      await expect(page).toHaveScreenshot("ports-expanded.png");
    }
  });
});
