import { test, expect } from "@playwright/test";
import {
  emptyState,
  singleRunningPod,
  singleStoppedPod,
  singleErrorPod,
  startingPod,
  mixedPods,
  manyPods,
  longNamePod,
  manyPortsPod,
  manyProcessesPod,
} from "../fixtures/synthetic-data";
import { assertNoOverlap } from "../helpers/layout-assertions";

async function injectPods(page: any, pods: any[]) {
  await page.evaluate(async (pods: any[]) => {
    await (window as any).__TAURI__.core.invoke("test_inject_pods", { pods });
  }, pods);
  // Allow UI to update
  await page.waitForTimeout(500);
}

test.describe("Pod tiles", () => {
  test("empty state shows placeholder", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await injectPods(page, emptyState());
    await expect(page.getByTestId("empty-state")).toBeVisible();
    await expect(page).toHaveScreenshot("empty-state.png");
  });

  test("single running pod", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await injectPods(page, singleRunningPod());
    await expect(page.getByTestId("pod-tile-pod-running-1")).toBeVisible();
    await expect(page).toHaveScreenshot("single-running.png");
  });

  test("single stopped pod", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await injectPods(page, singleStoppedPod());
    await expect(page.getByTestId("pod-tile-pod-stopped-1")).toBeVisible();
    await expect(page).toHaveScreenshot("single-stopped.png");
  });

  test("single error pod", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await injectPods(page, singleErrorPod());
    await expect(page.getByTestId("pod-tile-pod-error-1")).toBeVisible();
    await expect(page).toHaveScreenshot("single-error.png");
  });

  test("starting pod shows spinner", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await injectPods(page, startingPod());
    await expect(page.getByTestId("pod-tile-pod-starting-1")).toBeVisible();
    await expect(page).toHaveScreenshot("starting-pod.png");
  });

  test("mixed running + stopped pods", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await injectPods(page, mixedPods());
    await assertNoOverlap(page, '[data-testid^="pod-tile"]');
    await expect(page).toHaveScreenshot("mixed-pods.png");
  });

  test("50 pods with scroll", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await injectPods(page, manyPods(50));
    await assertNoOverlap(page, '[data-testid^="pod-tile"]');
    await expect(page).toHaveScreenshot("many-pods.png");
  });

  test("long pod name with ellipsis", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await injectPods(page, longNamePod());
    await expect(page.getByTestId("pod-tile-pod-long-name")).toBeVisible();
    await expect(page).toHaveScreenshot("long-name.png");
  });

  test("pod with 15 ports", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await injectPods(page, manyPortsPod());
    await expect(page.getByTestId("pod-tile-pod-many-ports")).toBeVisible();
    await expect(page).toHaveScreenshot("many-ports.png");
  });

  test("pod with 30 processes", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await injectPods(page, manyProcessesPod());
    await expect(page.getByTestId("pod-tile-pod-many-procs")).toBeVisible();
    await expect(page).toHaveScreenshot("many-processes.png");
  });
});
