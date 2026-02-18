#!/usr/bin/env node
// E2E smoke test: devcontainer lifecycle + port detection + exposure
import { execSync, spawn } from "child_process";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";
import http from "http";
import net from "net";

const __dirname = dirname(fileURLToPath(import.meta.url));
const FIXTURE_DIR = resolve(__dirname, "e2e/fixtures/test-devcontainer");

let containerId = null;
let proxyServer = null;

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

function exec(cmd, opts = {}) {
  return execSync(cmd, { encoding: "utf-8", timeout: 120_000, ...opts }).trim();
}

function httpGet(url, timeout = 5000) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error("timeout")), timeout);
    http
      .get(url, (res) => {
        let data = "";
        res.on("data", (c) => (data += c));
        res.on("end", () => {
          clearTimeout(timer);
          resolve({ status: res.statusCode, body: data });
        });
      })
      .on("error", (e) => {
        clearTimeout(timer);
        reject(e);
      });
  });
}

function createTcpProxy(hostPort, targetIp, targetPort) {
  return new Promise((resolve, reject) => {
    const server = net.createServer((client) => {
      const target = net.connect(targetPort, targetIp);
      client.pipe(target);
      target.pipe(client);
      client.on("error", () => target.destroy());
      target.on("error", () => client.destroy());
    });
    server.listen(hostPort, "127.0.0.1", () => resolve(server));
    server.on("error", reject);
  });
}

async function cleanup() {
  console.log("\n--- CLEANUP ---");
  if (proxyServer) {
    proxyServer.close();
    proxyServer = null;
  }
  if (containerId) {
    try {
      exec(`docker stop ${containerId}`);
    } catch {}
    try {
      exec(`docker rm -f ${containerId}`);
    } catch {}
  }
  // Clean by label
  try {
    const ids = exec(
      `docker ps -a --filter "label=devcontainer.local_folder=${FIXTURE_DIR}" -q`
    );
    if (ids) exec(`docker rm -f ${ids.split("\n").join(" ")}`);
  } catch {}
  console.log("Cleanup done");
}

async function main() {
  try {
    // ========== STEP 1: Start devcontainer ==========
    console.log("=== STEP 1: Start devcontainer ===");
    console.log(`Fixture: ${FIXTURE_DIR}`);

    const upOutput = exec(`devcontainer up --workspace-folder "${FIXTURE_DIR}" 2>&1`, {
      timeout: 180_000,
    });
    console.log(upOutput.split("\n").slice(-3).join("\n"));

    // Extract container ID and port from devcontainer up output
    const idMatch = upOutput.match(/"containerId":"([^"]+)"/);
    if (idMatch) {
      containerId = idMatch[1];
    } else {
      containerId = exec(
        `docker ps --filter "label=devcontainer.local_folder=${FIXTURE_DIR}" -q`
      )
        .split("\n")[0]
        .trim();
    }
    if (!containerId) throw new Error("Could not find container ID");
    console.log(`Container ID: ${containerId.substring(0, 12)}`);

    // Extract port from devcontainer up output (postStartCommand prints it)
    const portFromUp = upOutput.match(/NOOK_TEST_PORT=(\d+)/);
    const earlyPort = portFromUp ? parseInt(portFromUp[1], 10) : null;
    if (earlyPort) console.log(`Port from devcontainer up output: ${earlyPort}`);

    // ========== STEP 2: Verify running ==========
    console.log("\n=== STEP 2: Verify container is running ===");
    const status = exec(`docker inspect -f '{{.State.Status}}' ${containerId}`);
    console.log(`Status: ${status}`);
    if (status !== "running") throw new Error(`Container not running: ${status}`);
    console.log("PASS: Container is running");

    // ========== STEP 3: Detect port ==========
    console.log("\n=== STEP 3: Detect server port ===");
    let detectedPort = earlyPort;

    if (!detectedPort) {
      // Try docker logs
      for (let attempt = 0; attempt < 10; attempt++) {
        await sleep(2000);
        try {
          const logs = exec(`docker logs ${containerId} 2>&1`);
          const portMatch = logs.match(/NOOK_TEST_PORT=(\d+)/);
          if (portMatch) {
            detectedPort = parseInt(portMatch[1], 10);
            break;
          }
        } catch {}
        console.log(`  Waiting for server... (attempt ${attempt + 1})`);
      }
    }

    if (!detectedPort) {
      // Fallback: use ss inside container to find listening node process
      await sleep(3000);
      try {
        const ssOut = exec(`docker exec ${containerId} ss -tlnp 2>/dev/null`);
        const nodeMatch = ssOut.match(/:(\d+)\s.*node/);
        if (nodeMatch) detectedPort = parseInt(nodeMatch[1], 10);
      } catch {}
    }

    if (!detectedPort) throw new Error("Server port not detected");
    console.log(`Detected port: ${detectedPort}`);
    console.log("PASS: Port detected");

    // ========== STEP 4: Verify port inside container ==========
    console.log("\n=== STEP 4: Verify port listening inside container (ss -tlnp) ===");
    let ssOutput;
    try {
      ssOutput = exec(`docker exec ${containerId} ss -tlnp`);
    } catch {
      ssOutput = "ss not available";
    }

    if (ssOutput.includes(`:${detectedPort}`)) {
      console.log(`PASS: Port ${detectedPort} confirmed via ss`);
    } else {
      // Fallback: /proc/net/tcp
      const hexPort = detectedPort.toString(16).toUpperCase().padStart(4, "0");
      try {
        const procTcp = exec(`docker exec ${containerId} cat /proc/net/tcp`);
        if (procTcp.toUpperCase().includes(`:${hexPort}`)) {
          console.log(`PASS: Port ${detectedPort} confirmed via /proc/net/tcp`);
        } else {
          console.log("WARN: Port not confirmed via ss or /proc/net/tcp");
        }
      } catch {
        console.log("WARN: Could not check /proc/net/tcp");
      }
    }

    // ========== STEP 5: Expose port to host (TCP proxy) ==========
    console.log("\n=== STEP 5: Expose port to host ===");
    const containerIp = exec(
      `docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' ${containerId}`
    );
    if (!containerIp) throw new Error("Could not get container IP");
    console.log(`Container IP: ${containerIp}`);

    // Pick free host port
    let hostPort = detectedPort + 1000;
    while (true) {
      try {
        exec(`lsof -ti:${hostPort}`);
        hostPort++;
      } catch {
        break; // Port is free
      }
    }

    console.log(`Forwarding localhost:${hostPort} -> ${containerIp}:${detectedPort}`);
    proxyServer = await createTcpProxy(hostPort, containerIp, detectedPort);
    await sleep(500);
    console.log("PASS: TCP proxy started");

    // ========== STEP 6: Verify HTTP access ==========
    console.log("\n=== STEP 6: Verify HTTP access through exposed port ===");
    const resp = await httpGet(`http://localhost:${hostPort}/`);
    console.log(`Response: ${resp.body}`);
    const json = JSON.parse(resp.body);
    if (json.status !== "ok") throw new Error(`Unexpected status: ${json.status}`);
    if (json.port !== detectedPort)
      throw new Error(`Port mismatch: ${json.port} vs ${detectedPort}`);
    console.log("PASS: HTTP response received through exposed port");

    const health = await httpGet(`http://localhost:${hostPort}/health`);
    console.log(`Health: ${health.body}`);
    const healthJson = JSON.parse(health.body);
    if (!healthJson.healthy) throw new Error("Health check failed");
    console.log("PASS: Health endpoint works");

    // ========== STEP 7: Unexpose (stop proxy) and verify ==========
    console.log("\n=== STEP 7: Unexpose port ===");
    await new Promise((resolve) => {
      proxyServer.close(() => resolve());
      // Force-close all existing connections
      proxyServer.unref();
    });
    proxyServer = null;
    await sleep(1500);

    let portStillOpen = false;
    try {
      await httpGet(`http://localhost:${hostPort}/`, 2000);
      portStillOpen = true;
    } catch {
      portStillOpen = false;
    }
    if (portStillOpen) throw new Error("Port still accessible after unexpose");
    console.log("PASS: Port no longer accessible after unexpose");

    // ========== STEP 8: Stop devcontainer ==========
    console.log("\n=== STEP 8: Stop devcontainer ===");
    try {
      exec(`devcontainer stop --workspace-folder "${FIXTURE_DIR}"`, {
        timeout: 30_000,
      });
    } catch {
      exec(`docker stop ${containerId}`);
    }
    await sleep(2000);

    let postStatus;
    try {
      postStatus = exec(`docker inspect -f '{{.State.Status}}' ${containerId}`);
    } catch {
      postStatus = "removed";
    }
    console.log(`Status after stop: ${postStatus}`);
    if (postStatus === "exited" || postStatus === "removed") {
      console.log("PASS: Container stopped");
    } else {
      console.log(`WARN: Unexpected status '${postStatus}', force stopping...`);
      exec(`docker stop ${containerId}`);
    }

    // ========== STEP 9: Remove container ==========
    console.log("\n=== STEP 9: Remove container ===");
    try {
      exec(`docker rm -f ${containerId}`);
    } catch {}
    let exists = true;
    try {
      exec(`docker inspect ${containerId}`);
    } catch {
      exists = false;
    }
    if (exists) throw new Error("Container still exists after rm");
    console.log("PASS: Container removed");
    containerId = null;

    console.log("\n=========================================");
    console.log("  ALL E2E SMOKE TESTS PASSED");
    console.log("=========================================");
  } catch (e) {
    console.error(`\nFAIL: ${e.message}`);
    await cleanup();
    process.exit(1);
  }

  await cleanup();
}

main();
