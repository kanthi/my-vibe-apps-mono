# Security audit

Audit of `apps/freedium-browser-ext` on 2026-09-28. Scope is the extension source in `shared/` and the three manifests. The mirror site itself was not reviewed. Safari was not run, because this Mac does not have Xcode's converter. Firefox was not loaded in a browser.

## What it does

The extension does nothing until you click the toolbar button, use the shortcut, or choose the context-menu item. It then navigates a tab to:

```text
https://freedium-mirror.cfd/<page address>
```

That is the whole product. The mirror operator receives the address you chose to open.

## Permissions

Each manifest asks only for:

| Permission | Why |
|---|---|
| `activeTab` | Read the current tab's URL after you invoke the extension. |
| `contextMenus` | Add "Open on Freedium" to the page, link, and toolbar menus. |
| `storage` | Remember the mirror address in `storage.local`. |

There is no `<all_urls>`, no host permission, no `cookies`, no `webRequest`, no `scripting`, and no `tabs` permission that would expose every open tab without a click. Web pages cannot message the extension: there is no `externally_connectable` and no `onMessage` listener. The options page and the service worker load only files packed in the extension. There is no `eval`, no inline script, and no remote script. `npm` dependencies: none.

The extension does not read page contents, cookies, or the password store. The only address it sees is the one the browser passes in when you invoke it.

## Fixes applied in this audit

1. **Fragments are stripped.** A fragment such as `#access_token=…` is not sent to the server, but it would be readable by script on the mirror page. `toFreediumUrl` now drops it. The mirror does not need it to fetch an article.

2. **Usernames and passwords are stripped.** `https://user:secret@example.com/a` is sent as `https://example.com/a`. A mirror URL that itself contains a username or password is rejected.

3. **The result must stay on the mirror origin.** After the page address is joined on, the code parses the result and refuses it unless the origin is exactly the configured mirror. That blocks a joined string from being reparsed as a different host.

4. **Remote http mirrors are rejected.** An `http://` mirror would send the page address in clear text. `https` is required. `http` is accepted only for `localhost`, `127.0.0.1`, and `[::1]`, so a future local mirror can still be used. A stored mirror that no longer passes this check falls back to `https://freedium-mirror.cfd`.

5. **Extension pages declare a content security policy** of `script-src 'self'; object-src 'self'`. Manifest V3 already defaults to this. Stating it keeps a later edit from loosening it quietly.

## Accepted risks

**The path and query string are sent to the mirror.** That includes article ids and also anything else in the query string: session ids, magic-login links, email addresses. This matches the public Freedium bookmarklet. Stripping every query string would break pages that need one. Do not use the button on a URL you would not hand to that site.

**You can point the mirror at any https origin.** Options stores that origin and the next click sends the page address there. The field is yours. A typo sends the address to the wrong host.

**The mirror is a third party.** This extension does not check what `freedium-mirror.cfd` returns, stores, or injects. Once the tab navigates there, that site's scripts run with the usual powers of a website you opened. The extension does not keep running inside that page.

**A right-clicked link is sent too.** The menu item "Open on Freedium" on a link opens that link's URL, not only the page you are reading. It still has to be `http` or `https`. `javascript:`, `data:`, `blob:`, `file:`, and browser-internal URLs are refused.

**The build is unsigned.** Chrome, Brave, and the other Chromium browsers load `chrome-based` as an unpacked extension. Firefox's load is a temporary add-on until it is signed and published. Anyone who can change the files on disk can change what the extension does. That is true of every unpacked extension.

**`storage.local` is not a secret store.** The mirror origin saved there is not encrypted by this extension. It is not a credential.

## What was reviewed

- `shared/src/rewrite.js`, `background.js`, `options.js`, `options.html`, `options.css`
- `chrome-based/manifest.json`, `firefox/manifest.json`, `safari/manifest.json`
- `scripts/sync.js`, `scripts/safari-xcode.sh`, `scripts/make-icons.py`

`options.js` writes status with `textContent` and assigns the mirror with an input's `value`. It does not build HTML from the stored string. `background.js` navigates only to the string `toFreediumUrl` returns.

Tests in `test/rewrite.test.js` cover the fragment, credential, origin, and mirror-scheme checks. `npm test` also checks that the three browser folders match `shared/` and that none of the manifests gained a broader permission list.
