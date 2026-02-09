const https = require("https");
const fs = require("fs");
const path = require("path");
const os = require("os");

const VERSION = "v0.1.0";
const REPO = "rsolizPL/sails-decl-rs";

function targetTriple() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === "darwin" && arch === "x64") return "x86_64-apple-darwin";
  if (platform === "darwin" && arch === "arm64") return "aarch64-apple-darwin";
  if (platform === "linux" && arch === "x64") return "x86_64-unknown-linux-gnu";
  if (platform === "linux" && arch === "arm64") return "aarch64-unknown-linux-gnu";
  if (platform === "win32" && arch === "x64") return "x86_64-pc-windows-msvc";

  throw new Error(`Unsupported platform: ${platform} ${arch}`);
}

function binaryName() {
  return process.platform === "win32" ? "sails-decl-rs.exe" : "sails-decl-rs";
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    https.get(url, res => {
      if (res.statusCode !== 200) {
        reject(new Error(`Download failed: ${res.statusCode}`));
        return;
      }

      fs.mkdirSync(path.dirname(dest), { recursive: true });
      const file = fs.createWriteStream(dest);
      res.pipe(file);
      file.on("finish", () => file.close(resolve));
    }).on("error", reject);
  });
}

(async () => {
  try {
    const triple = targetTriple();
    const name = binaryName();
    const url = `https://github.com/${REPO}/releases/download/${VERSION}/${name}-${triple}`;
    const out = path.join(__dirname, "native", name);

    console.log(`Downloading sails-decl-rs (${triple})...`);
    await download(url, out);

    if (process.platform !== "win32") {
      fs.chmodSync(out, 0o755);
    }

    console.log("sails-decl-rs installed successfully");
  } catch (err) {
    console.error(err.message);
    process.exit(1);
  }
})();