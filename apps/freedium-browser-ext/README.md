# Freedium

A browser extension that opens the current page on [freedium-mirror.cfd](https://freedium-mirror.cfd/).

Status: `usable`.

Click the toolbar button and the tab goes to:

```text
https://freedium-mirror.cfd/<the page you were on>
```

Back returns to the original page. It works for any `http` or `https` page. Browser pages (`chrome://`, `brave://`, `about:`) stay put, and the button shows a short dash when it has nothing to open.

Usernames, passwords, and URL fragments are removed before the address is sent. The path and query string are sent. The security write-up is [SECURITY.md](SECURITY.md).

The source of truth is `shared/`. `npm run sync` copies it into the three browser folders. Each folder is a complete extension.

## Install

Brave is already pointed at the old folder. Load `chrome-based` again so it picks up this layout.

### Chrome, Brave, Edge, Arc, Opera, Vivaldi

1. Open the browser's extensions page (`chrome://extensions` or `brave://extensions`).
2. Turn on Developer mode.
3. Choose **Load unpacked** and select `chrome-based` (the folder that contains that copy of `manifest.json`).
4. Pin **Freedium** if you want the button in the toolbar.

### Firefox

1. Open `about:debugging#/runtime/this-firefox`.
2. Choose **Load Temporary Add-on**.
3. Select `firefox/manifest.json`.

A temporary add-on is removed when Firefox quits. The manifest id is `freedium-browser-ext@kanthi.github.io`. Firefox 128 or newer is required.

### Safari

Safari loads the extension from a Mac app. Full Xcode provides the converter (the Command Line Tools do not):

```bash
./scripts/safari-xcode.sh
```

That wraps `safari/` and writes an Xcode project under `safari-xcode/`. Open it, run it once, and enable the extension in Safari → Settings → Extensions. Safari 16.4 or newer is required.

## Use

- Click the toolbar icon. The current tab opens on the mirror.
- Keyboard shortcut: Ctrl+Shift+F, or Control+Shift+F on a Mac. Change it in the browser’s keyboard shortcuts for extensions.
- Right-click the page and choose **Open on Freedium**.
- Right-click a link to open that link on the mirror in a new tab.

The mirror address lives in the extension’s options. The default is `https://freedium-mirror.cfd`. A custom mirror must be https, except `localhost`, `127.0.0.1`, and `[::1]`.

## Develop

Edit files under `shared/`, then sync:

```bash
npm test
python3 scripts/make-icons.py   # redraw shared/icons, then npm run sync
```

`npm test` copies `shared/` into `chrome-based`, `firefox`, and `safari` before the tests run. After editing, reload the extension in the browser.

This extension only rewrites the address you choose to open. It is not affiliated with Freedium or with the sites you open.
