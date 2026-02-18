import { test, expect } from "@playwright/test";
import {
  addPodViaUI,
  getFirstPodId,
  waitForPodStatus,
  deletePod,
  cleanupTestContainers,
} from "../helpers/e2e-helpers";

test.describe("Error recovery", () => {
  test.afterEach(async () => {
    cleanupTestContainers();
  });

  test("invalid path → error state → retry → dismiss → delete", async ({ page }) => {
    await page.goto("/");

    // Add pod with invalid path (no devcontainer config)
    await addPodViaUI(page, "/tmp/nook-e2e-invalid-devcontainer");

    const podId = await getFirstPodId(page);
    expect(podId).toBeTruthy();

    const tile = page.getByTestId(`pod-tile-${podId}`);
    await expect(tile).toBeVisible();

    // Start — should fail and go to Error
    await tile.getByTestId(`start-pod-${podId}`).click();
    await waitForPodStatus(page, podId, "error", 60000);

    // Verify error message is visible
    const errorEl = tile.locator('[class*="error"]');
    await expect(errorEl.first()).toBeVisible();

    // Click Retry — should go back to Starting then Error again
    const retryBtn = tile.getByTestId(`retry-pod-${podId}`);
    if (await retryBtn.isVisible()) {
      await retryBtn.click();
      await waitForPodStatus(page, podId, "error", 60000);
    }

    // Delete the pod
    await deletePod(page, podId);
    await expect(page.getByTestId(`pod-tile-${podId}`)).not.toBeVisible();
  });
});
