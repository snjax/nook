import { test, expect } from "@playwright/test";
import {
  TEST_FIXTURE_PATH,
  addPodViaUI,
  getFirstPodId,
  startPodAndWaitRunning,
  stopPodAndWaitStopped,
  deletePod,
  cleanupTestContainers,
} from "../helpers/e2e-helpers";

test.describe("Pod full lifecycle", () => {
  test.afterEach(async () => {
    cleanupTestContainers();
  });

  test("add → start → verify running → stop → delete", async ({ page }) => {
    await page.goto("/");

    // Add pod from fixture path
    await addPodViaUI(page, TEST_FIXTURE_PATH);

    // Verify pod tile appears
    const podId = await getFirstPodId(page);
    expect(podId).toBeTruthy();

    const tile = page.getByTestId(`pod-tile-${podId}`);
    await expect(tile).toBeVisible();

    // Start and wait for Running (180s timeout for first build)
    await startPodAndWaitRunning(page, podId, 180000);

    // Stop and wait for Stopped
    await stopPodAndWaitStopped(page, podId, 60000);

    // Delete pod
    await deletePod(page, podId);

    // Verify tile is gone
    await expect(page.getByTestId(`pod-tile-${podId}`)).not.toBeVisible();
  });
});
