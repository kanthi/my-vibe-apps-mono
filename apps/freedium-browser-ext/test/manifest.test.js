const test = require("node:test");
const assert = require("node:assert/strict");
const crypto = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");

const root = path.join(__dirname, "..");
const targets = ["chrome-based", "firefox", "safari"];

function sha256(file) {
  return crypto
    .createHash("sha256")
    .update(fs.readFileSync(file))
    .digest("hex");
}

test("each browser folder is a loadable extension", () => {
  for (const target of targets) {
    const dir = path.join(root, target);
    const manifest = JSON.parse(
      fs.readFileSync(path.join(dir, "manifest.json"), "utf8"),
    );
    assert.equal(manifest.manifest_version, 3, target);
    assert.equal(manifest.name, "Freedium", target);
    assert.deepEqual(manifest.permissions, [
      "activeTab",
      "contextMenus",
      "storage",
    ]);
    assert.equal(
      manifest.content_security_policy.extension_pages,
      "script-src 'self'; object-src 'self'",
    );
    const files = [
      manifest.background.service_worker,
      manifest.options_ui.page,
      "src/options.css",
      "src/options.js",
      "src/rewrite.js",
      ...Object.values(manifest.icons),
    ];
    for (const file of files) {
      assert.equal(
        fs.existsSync(path.join(dir, file)),
        true,
        `${target} missing ${file}`,
      );
    }
  }
});

test("firefox has a stable add-on id and safari requires 16.4", () => {
  const firefox = JSON.parse(
    fs.readFileSync(path.join(root, "firefox/manifest.json"), "utf8"),
  );
  assert.equal(
    firefox.browser_specific_settings.gecko.id,
    "freedium-browser-ext@kanthi.github.io",
  );
  assert.equal(
    firefox.browser_specific_settings.gecko.strict_min_version,
    "128.0",
  );
  const safari = JSON.parse(
    fs.readFileSync(path.join(root, "safari/manifest.json"), "utf8"),
  );
  assert.equal(safari.browser_specific_settings.safari.strict_min_version, "16.4");
  const chrome = JSON.parse(
    fs.readFileSync(path.join(root, "chrome-based/manifest.json"), "utf8"),
  );
  assert.equal(chrome.browser_specific_settings, undefined);
});

test("browser folders match shared source and icons", () => {
  for (const name of ["src", "icons"]) {
    const sharedDir = path.join(root, "shared", name);
    for (const file of fs.readdirSync(sharedDir)) {
      const sharedFile = path.join(sharedDir, file);
      if (!fs.statSync(sharedFile).isFile()) continue;
      for (const target of targets) {
        const copy = path.join(root, target, name, file);
        assert.equal(sha256(copy), sha256(sharedFile), `${target}/${name}/${file}`);
      }
    }
  }
});
