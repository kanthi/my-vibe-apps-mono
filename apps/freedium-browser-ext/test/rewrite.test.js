const test = require("node:test");
const assert = require("node:assert/strict");
const {
  DEFAULT_MIRROR,
  normalizeMirror,
  canMirror,
  isMirrorPage,
  toFreediumUrl,
} = require("../shared/src/rewrite.js");

const article = "https://medium.com/@ada/a-long-note-on-reading-1a2b3c4d5e6f";

test("prefixes the page URL with the default mirror", () => {
  const target = toFreediumUrl(article);
  assert.equal(target, `${DEFAULT_MIRROR}/${article}`);
  assert.equal(new URL(target).origin, DEFAULT_MIRROR);
});

test("keeps the query string on the page URL", () => {
  const page = "https://example.com/story?id=7&ref=share";
  assert.equal(toFreediumUrl(page), `${DEFAULT_MIRROR}/${page}`);
});

test("accepts http pages", () => {
  const page = "http://example.com/a";
  assert.equal(canMirror(page), true);
  assert.equal(toFreediumUrl(page), `${DEFAULT_MIRROR}/${page}`);
});

test("drops fragments so the mirror page cannot read them", () => {
  assert.equal(
    toFreediumUrl("https://example.com/a#access_token=secret"),
    `${DEFAULT_MIRROR}/https://example.com/a`,
  );
});

test("drops usernames and passwords from the page URL", () => {
  assert.equal(
    toFreediumUrl("https://user:secret@example.com/a?x=1"),
    `${DEFAULT_MIRROR}/https://example.com/a?x=1`,
  );
});

test("refuses browser-internal and empty URLs", () => {
  for (const page of [
    "chrome://extensions/",
    "brave://settings/",
    "about:blank",
    "file:///tmp/a.html",
    "javascript:alert(1)",
    "data:text/html,hi",
    "blob:https://example.com/uuid",
    "chrome-extension://abcdef/page.html",
    "",
    "not a url",
  ]) {
    assert.equal(canMirror(page), false, page);
    assert.equal(toFreediumUrl(page), null, page);
  }
});

test("does not wrap a page that is already on the mirror", () => {
  const page = `${DEFAULT_MIRROR}/${article}`;
  assert.equal(isMirrorPage(page), true);
  assert.equal(toFreediumUrl(page), null);
});

test("normalizes a custom https mirror and uses it as the prefix", () => {
  assert.equal(
    normalizeMirror("https://example.test/ignored/"),
    "https://example.test",
  );
  assert.equal(normalizeMirror("mirror.example"), "https://mirror.example");
  const page = "https://nytimes.com/2026/01/01/a.html";
  assert.equal(
    toFreediumUrl(page, "https://example.test/"),
    `https://example.test/${page}`,
  );
});

test("allows http only for a local mirror", () => {
  assert.equal(normalizeMirror("http://localhost:8080"), "http://localhost:8080");
  assert.equal(normalizeMirror("http://127.0.0.1:9"), "http://127.0.0.1:9");
  assert.equal(normalizeMirror("http://[::1]:8080"), "http://[::1]:8080");
  assert.throws(() => normalizeMirror("http://example.com"), /https/);
  assert.equal(toFreediumUrl(article, "http://example.com"), null);
});

test("rejects a mirror that is not http(s) or that embeds credentials", () => {
  assert.throws(() => normalizeMirror("ftp://files.example"), /https/);
  assert.throws(
    () => normalizeMirror("https://user:pass@example.test"),
    /password/,
  );
  assert.equal(toFreediumUrl(article, "ftp://files.example"), null);
  assert.equal(toFreediumUrl(article, "https://user:pass@example.test"), null);
});
