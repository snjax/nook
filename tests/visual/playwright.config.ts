import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./specs",
  snapshotDir: "./snapshots",
  snapshotPathTemplate: "{snapshotDir}/{testFilePath}/{arg}{ext}",

  expect: {
    toHaveScreenshot: {
      maxDiffPixelRatio: 0.001,
      threshold: 0.2,
      animations: "disabled",
    },
  },

  projects: [
    { name: "hd", use: { viewport: { width: 1280, height: 720 } } },
    { name: "fhd", use: { viewport: { width: 1920, height: 1080 } } },
    { name: "qhd", use: { viewport: { width: 2560, height: 1440 } } },
    { name: "4k", use: { viewport: { width: 3840, height: 2160 } } },
  ],

  webServer: {
    command: "cargo tauri dev --features test-api",
    port: 1420,
    reuseExistingServer: true,
    timeout: 120000,
  },
});
