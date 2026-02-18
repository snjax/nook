import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./specs",
  timeout: 180000,
  retries: 0,
  workers: 1,

  expect: {
    timeout: 15000,
  },

  globalSetup: require.resolve("./global-setup"),
  globalTeardown: require.resolve("./global-teardown"),

  projects: [
    {
      name: "e2e",
      use: {
        viewport: { width: 1280, height: 720 },
        baseURL: "http://localhost:1420",
      },
    },
  ],

  webServer: {
    command: "cargo tauri dev",
    port: 1420,
    reuseExistingServer: true,
    timeout: 180000,
  },
});
