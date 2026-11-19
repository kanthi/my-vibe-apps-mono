# Homebrew Cask (macOS)

> **Status: not currently available.** There is **no working Homebrew tap for this fork**. The original
> `khoopapp/khoop` tap referenced by older releases is gone — the upstream `khoopapp` GitHub org (and its
> `homebrew-khoop` repo) no longer exists (#1069), so `brew tap khoopapp/khoop` fails outright.
>
> **Install Khoop on macOS by downloading `Khoop.app` directly from
> [Releases](https://github.com/ryanbr/khoop/releases)** — Apple Silicon + Intel, drag to Applications.
> See **First launch on macOS** in the [README](../README.md#download) for the one-time Gatekeeper step
> (Khoop ships anonymously and isn't notarized, so macOS blocks it on first open until you clear the
> download quarantine flag).

The rest of this doc is a maintainer reference for **re-publishing** a tap under this fork, should that
happen later. Until a tap exists and this notice is removed, ignore any `brew …` commands below.

## Publishing a tap under this fork (maintainers)

A Homebrew cask lets macOS users install + auto-update with:

```bash
brew tap <org>/khoop
brew trust <org>/khoop      # required since Homebrew 6.0.0 (see note below)
brew install --cask khoop
brew upgrade --cask khoop   # later updates
```

To bring that back, a public `homebrew-khoop` tap repo must be created under this fork's org (e.g.
`ryanbr/homebrew-khoop`), holding `Casks/khoop.rb` pointing at the macOS `.zip` attached to each release.
`Tools/update-homebrew-cask.sh` still automates the cask refresh, but it defaults to the dead `KhoopApp`
tap — point it at the new repo via the `FORGE_ORG` / `FORGE_REPO` environment variables:

```bash
FORGE_ORG=ryanbr Tools/update-homebrew-cask.sh <version>   # e.g. … 1.95
```

That script computes the release zip's SHA256, regenerates `Casks/khoop.rb`, and pushes it to the tap.
There is **no GitHub Actions workflow / repo secret** — releases are cut by hand, so the cask update
rides along with them.

> **Why `brew trust`?** Since **Homebrew 6.0.0** (June 2026), non-official taps must be explicitly
> trusted before Homebrew will load their code — otherwise you'll see
> `Error: Refusing to load cask <org>/khoop/khoop from untrusted tap`. Trust is a one-time, per-machine
> decision (publishers can't pre-trust their own tap — only Homebrew's official taps are trusted by
> default). Trust the whole tap with `brew trust <org>/khoop`, or just the cask with
> `brew trust --cask <org>/khoop/khoop`. It's the Homebrew equivalent of the Gatekeeper right-click-Open:
> you're vouching for code you can read — the cask is one short file in the public tap, and the app's
> full source is in this repo.

> **Unsigned-app note.** Khoop ships anonymously with no Apple Developer ID, so it isn't notarized.
> Homebrew can't strip the quarantine flag for an un-notarized app, so on **first launch** Gatekeeper
> blocks it. On **macOS 15 Sequoia and later**: try to open Khoop once, then **System Settings →
> Privacy & Security**, scroll down, and click **"Open Anyway"** next to Khoop. (On macOS 14 and
> earlier you can right-click Khoop in `/Applications` → **Open** → **Open**.) Updates after that are
> just `brew upgrade`.

## Requirements (for a republished tap)

- A public `homebrew-khoop` tap repo exists under the fork's org.
- A PAT with **Contents: Read and write** on that repo (the same one used to push releases), read from a
  local file and supplied through a transient git credential helper, so **the token never appears on a
  command line, in a remote URL, or in any output**.

## Anonymity checklist

- Tap repo + commits under an anonymous identity (the script commits with a configurable name/email).
- Token read from the local file only; never echoed. Scope it to the repos it needs and no more.
- The cask installs the **already-anonymized** release zip (scrubbed by `Tools/anonymize-macos-app.sh`
  at build time) — no new surface.
