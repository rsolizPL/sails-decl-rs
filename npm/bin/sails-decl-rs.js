#!/usr/bin/env node

const { spawnSync } = require("child_process");
const path = require("path");
const fs = require("fs");

function getBinaryPath() {
  const base = path.join(__dirname, "..", "native");
  const exe = process.platform === "win32" ? "sails-decl-rs.exe" : "sails-decl-rs";
  return path.join(base, exe);
}

const binPath = getBinaryPath();

if (!fs.existsSync(binPath)) {
  console.error("sails-decl-rs binary not found. Try reinstalling the package.");
  process.exit(1);
}

const result = spawnSync(binPath, process.argv.slice(2), {
  stdio: "inherit"
});

process.exit(result.status ?? 1);