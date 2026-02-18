import { Page, expect } from "@playwright/test";
import * as path from "path";
import { execSync } from "child_process";

// Absolute path to the test devcontainer fixture
export const TEST_FIXTURE_PATH = path.resolve(__dirname, "../fixtures/test-devcontainer");

// Add a pod via UI - clicks "Add Pod" button, types path into dialog, submits
export async function addPodViaUI(page: Page, projectPath: string): Promise<void> {
  await page.getByTestId("add-pod-button").click();
  // Wait for dialog
  await page.getByTestId("add-pod-dialog").waitFor({ state: "visible" });
  // Type path
  await page.getByTestId("add-pod-path-input").fill(projectPath);
  // Submit
  await page.getByTestId("add-pod-submit").click();
  // Wait for dialog to close
  await page.getByTestId("add-pod-dialog").waitFor({ state: "hidden", timeout: 5000 });
}

// Wait for a pod tile to appear with a specific status
export async function waitForPodStatus(
  page: Page,
  podId: string,
  status: string,
  timeout: number = 60000,
): Promise<void> {
  await page.waitForFunction(
    ({ id, expectedStatus }) => {
      const tile = document.querySelector(`[data-testid="pod-tile-${id}"]`);
      if (!tile) return false;
      const statusEl = tile.querySelector('[data-testid^="pod-status"]');
      return statusEl?.textContent?.toLowerCase().includes(expectedStatus);
    },
    { id: podId, expectedStatus: status },
    { timeout },
  );
}

// Get the first pod tile's ID from the page
export async function getFirstPodId(page: Page): Promise<string> {
  const tile = page.locator('[data-testid^="pod-tile-"]').first();
  const testId = await tile.getAttribute("data-testid");
  return testId?.replace("pod-tile-", "") ?? "";
}

// Start a pod and wait for Running status
export async function startPodAndWaitRunning(
  page: Page,
  podId: string,
  timeout: number = 180000,
): Promise<void> {
  const tile = page.getByTestId(`pod-tile-${podId}`);
  await tile.getByTestId(`pod-start-${podId}`).click();
  await waitForPodStatus(page, podId, "running", timeout);
}

// Stop a pod and wait for Stopped status
export async function stopPodAndWaitStopped(
  page: Page,
  podId: string,
  timeout: number = 60000,
): Promise<void> {
  const tile = page.getByTestId(`pod-tile-${podId}`);
  await tile.getByTestId(`pod-stop-${podId}`).click();
  await waitForPodStatus(page, podId, "stopped", timeout);
}

// Delete a pod (clicks delete, confirms in dialog)
export async function deletePod(
  page: Page,
  podId: string,
): Promise<void> {
  const tile = page.getByTestId(`pod-tile-${podId}`);
  await tile.getByTestId(`pod-delete-${podId}`).click();
  // Wait for confirm dialog
  await page.getByTestId("delete-confirm-dialog").waitFor({ state: "visible" });
  await page.getByTestId("delete-confirm-button").click();
  // Wait for tile to disappear
  await page.getByTestId(`pod-tile-${podId}`).waitFor({ state: "hidden", timeout: 10000 });
}

// Force remove all test containers (cleanup)
export function cleanupTestContainers(): void {
  try {
    execSync(
      'docker ps -a --filter "label=devcontainer.local_folder" --format "{{.ID}}" | xargs -r docker rm -f',
      { stdio: "ignore", timeout: 30000 },
    );
  } catch {
    // Ignore errors during cleanup
  }
}

// Check if Docker is running
export function isDockerRunning(): boolean {
  try {
    execSync("docker info", { stdio: "ignore", timeout: 10000 });
    return true;
  } catch {
    return false;
  }
}
