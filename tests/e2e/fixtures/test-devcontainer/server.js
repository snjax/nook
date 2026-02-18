const express = require("express");
const app = express();

// Random port in range 10000-60000
const port = 10000 + Math.floor(Math.random() * 50000);

app.get("/", (req, res) => res.json({ status: "ok", port }));
app.get("/health", (req, res) => res.json({ healthy: true }));

app.listen(port, "0.0.0.0", () => {
  console.log(`NOOK_TEST_PORT=${port}`);
});
