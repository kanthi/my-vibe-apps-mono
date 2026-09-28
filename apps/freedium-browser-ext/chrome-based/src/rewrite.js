// URL helpers shared by the extension and the node tests.
(function (root, factory) {
  const api = factory();
  if (typeof module === "object" && module.exports) {
    module.exports = api;
  }
  root.Freedium = api;
})(typeof globalThis !== "undefined" ? globalThis : this, function () {
  const DEFAULT_MIRROR = "https://freedium-mirror.cfd";
  const LOCAL_MIRROR_HOSTS = new Set(["localhost", "127.0.0.1", "[::1]"]);

  function normalizeMirror(input) {
    const raw =
      input == null || String(input).trim() === ""
        ? DEFAULT_MIRROR
        : String(input).trim();
    const withScheme = /^[a-z][a-z0-9+.-]*:/i.test(raw) ? raw : `https://${raw}`;
    let url;
    try {
      url = new URL(withScheme);
    } catch {
      throw new Error("Mirror must be a valid URL");
    }
    if (url.username || url.password) {
      throw new Error("Mirror must not include a username or password");
    }
    if (url.protocol === "https:") {
      return url.origin;
    }
    if (
      url.protocol === "http:" &&
      LOCAL_MIRROR_HOSTS.has(url.hostname)
    ) {
      return url.origin;
    }
    throw new Error("Mirror must use https, or http on localhost");
  }

  function pageUrl(input) {
    if (typeof input !== "string" || input.trim() === "") return null;
    try {
      return new URL(input);
    } catch {
      return null;
    }
  }

  function canMirror(input) {
    const url = pageUrl(input);
    if (!url) return false;
    return url.protocol === "https:" || url.protocol === "http:";
  }

  // Drop credentials and the fragment. The fragment is visible to the mirror
  // page's JavaScript, and a username or password must not be forwarded.
  function publicPageHref(input) {
    const url = pageUrl(input);
    if (!url) return null;
    if (url.protocol !== "https:" && url.protocol !== "http:") return null;
    url.username = "";
    url.password = "";
    url.hash = "";
    return url.href;
  }

  function isMirrorPage(input, mirror) {
    const url = pageUrl(input);
    if (!url) return false;
    try {
      return url.origin === normalizeMirror(mirror);
    } catch {
      return false;
    }
  }

  // Mirror origin, then the page URL. The result must still be on that origin.
  function toFreediumUrl(input, mirror) {
    const page = publicPageHref(input);
    if (!page) return null;
    let base;
    try {
      base = normalizeMirror(mirror);
    } catch {
      return null;
    }
    if (isMirrorPage(page, base)) return null;
    let target;
    try {
      target = new URL(`${base}/${page}`);
    } catch {
      return null;
    }
    if (target.origin !== base || target.username || target.password) return null;
    return target.href;
  }

  return {
    DEFAULT_MIRROR,
    normalizeMirror,
    canMirror,
    isMirrorPage,
    toFreediumUrl,
  };
});
