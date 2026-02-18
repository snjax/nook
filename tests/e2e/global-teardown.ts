import { execSync } from "child_process";

export default async function globalTeardown() {
  console.log("[E2E Teardown] Cleaning up test containers...");
  try {
    execSync(
      'docker ps -a --filter "label=devcontainer.local_folder" --format "{{.ID}}" | xargs -r docker rm -f',
      { stdio: "ignore", timeout: 30000 },
    );
  } catch {
    // Ignore cleanup errors
  }
  console.log("[E2E Teardown] Done.");
}
