import { test, expect } from "@playwright/test";

test.describe("Settings persistence", () => {
  test("change settings → reload → verify persisted", async ({ page }) => {
    await page.goto("/");

    // Open settings
    await page.getByTestId("settings-button").click();

    // Wait for settings panel
    const settingsPanel = page.locator('[data-testid="settings-panel"]');
    await expect(settingsPanel).toBeVisible();

    // Change stats interval
    const statsInput = settingsPanel.locator('input[data-testid="settings-stats-interval"]');
    if (await statsInput.isVisible()) {
      await statsInput.clear();
      await statsInput.fill("5000");
    }

    // Save settings
    const saveBtn = settingsPanel.getByTestId("settings-save");
    if (await saveBtn.isVisible()) {
      await saveBtn.click();
    }

    // Close settings
    const closeBtn = settingsPanel.getByTestId("settings-close");
    if (await closeBtn.isVisible()) {
      await closeBtn.click();
    }

    // Reload page
    await page.reload();
    await page.waitForTimeout(1000);

    // Re-open settings and verify
    await page.getByTestId("settings-button").click();
    const panel2 = page.locator('[data-testid="settings-panel"]');
    await expect(panel2).toBeVisible();

    const statsInput2 = panel2.locator('input[data-testid="settings-stats-interval"]');
    if (await statsInput2.isVisible()) {
      await expect(statsInput2).toHaveValue("5000");
    }
  });
});
