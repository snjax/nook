import { execSync } from "child_process";

export default async function globalSetup() {
  console.log("[E2E Setup] Checking prerequisites...");

  // Verify Docker is running
  try {
    execSync("docker info", { stdio: "ignore", timeout: 10000 });
    console.log("[E2E Setup] ✓ Docker is running");
  } catch {
    throw new Error(
      "Docker is not running. Start Docker before running E2E tests.",
    );
  }

  // Verify devcontainer CLI
  try {
    execSync("devcontainer --version", { stdio: "ignore", timeout: 5000 });
    console.log("[E2E Setup] ✓ devcontainer CLI available");
  } catch {
    throw new Error(
      "devcontainer CLI not found. Install with: npm install -g @devcontainers/cli",
    );
  }

  // Remove stale test containers
  console.log("[E2E Setup] Cleaning up stale containers...");
  try {
    execSync(
      'docker ps -a --filter "label=devcontainer.local_folder" --format "{{.ID}}" | xargs -r docker rm -f',
      { stdio: "ignore", timeout: 30000 },
    );
  } catch {
    // Ignore cleanup errors
  }

  console.log("[E2E Setup] Ready.");
}
