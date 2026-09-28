// Copy shared/src and shared/icons into each browser folder.
const fs = require("node:fs");
const path = require("node:path");

const root = path.join(__dirname, "..");
const shared = path.join(root, "shared");
const targets = ["chrome-based", "firefox", "safari"];

function copyDir(from, to) {
  fs.mkdirSync(to, { recursive: true });
  for (const entry of fs.readdirSync(from, { withFileTypes: true })) {
    const src = path.join(from, entry.name);
    const dest = path.join(to, entry.name);
    if (entry.isDirectory()) copyDir(src, dest);
    else fs.copyFileSync(src, dest);
  }
}

for (const target of targets) {
  copyDir(path.join(shared, "src"), path.join(root, target, "src"));
  copyDir(path.join(shared, "icons"), path.join(root, target, "icons"));
}
