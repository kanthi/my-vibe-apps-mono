# Building Tanux on macOS

Tanux builds `noarch` RPMs and repacks a Linux ISO, so the packaging and ISO tools run inside a Fedora machine in [OrbStack](https://orbstack.dev) while all source files stay in this folder on the Mac.

## What you need

- An Apple Silicon Mac with OrbStack installed.
- About 4 GB of free disk for the builder container, cached upstream Everything ISO (~1.3 GB), and output ISO (~1.3 GB).
- Internet access to download the upstream ISO once and verify DNF5 package resolution.

## One-time setup

```sh
./scripts/setup-builder.sh
```

This creates an OrbStack machine named `tanux-builder` running Fedora 44 (`arm64`) and installs the lean RPM/ISO toolchain (`rpm-build`, `createrepo_c`, `xorriso`, `mtools`, `dosfstools`, `squashfs-tools`, `cpio`, `pigz`, `librsvg2-tools`, `pykickstart`, `isomd5sum`). OrbStack mounts your Mac home folder inside the machine at the same path.

Set `TANUX_BUILDER` in your shell to use a different OrbStack machine name.

## Check the editions

```sh
./scripts/check-kickstarts.sh            # all five editions
./scripts/check-kickstarts.sh kde sway   # subset
```

Rebuilds the local RPM repo (`build/repo/`), renders the Kickstarts from `installer/editions.py`, validates every Kickstart with `ksvalidator -v F45`, and dry-runs `dnf5 --releasever=45 install` against Fedora 45 mirrors plus `build/repo/` to catch missing packages, group renames, or RPM conflicts.

## Build the RPMs and ISO

```sh
./scripts/build-rpms.sh   # optional: builds only the 14 noarch RPMs into build/repo/
./scripts/build-iso.sh    # builds RPMs + writes build/tanux-45-aarch64.iso
```

What `scripts/build-iso.sh` does:

1. Downloads `Fedora-Everything-netinst-aarch64-45-1.4.iso` into `build/upstream/` if not cached and verifies its SHA-256 checksum against the official `CHECKSUM` file.
2. Runs `./scripts/build-rpms.sh` to compile all `rpm/*.spec` packages into `build/repo/` (`noarch`, `--define 'dist .fc45'`) and run `createrepo_c`.
3. Renders `/ks/*.ks`, `/EFI/BOOT/grub.cfg`, and the white-on-black GRUB theme from `installer/editions.py` and `installer/grub/theme.txt`, validating every Kickstart with `ksvalidator`.
4. Builds `images/product.img` (gzipped `cpio`) with `.buildstamp`, Tanux purple Anaconda GTK CSS/images, and Plymouth mark assets.
5. Injects the Tanux `grub.cfg` and theme into `images/efiboot.img` via `mtools` (no root or loop mounts needed).
6. Repacks the ISO with `xorriso` under volume ID `TANUX_45` and implants MD5 sums with `implantisomd5`.

`build/` is ignored by git. Rebuilding after any change takes only a few seconds once the upstream ISO is cached.

## Test the ISO in VMware Fusion

OrbStack containers share a single kernel and cannot boot an ISO. Use VMware Fusion (`arm64`):

```sh
./scripts/fusion-vm.sh         # create fresh 30 GB disk VM in build/vm/ and boot the newest ISO
./scripts/fusion-vm.sh disk    # detach ISO and boot the installed disk
./scripts/fusion-vm.sh stop    # power the VM off
```

The VM appears in VMware Fusion as "Tanux test" with 4 vCPUs, 4 GB RAM, a 30 GB NVMe disk, bridged `vmxnet3` networking, and the ISO attached as a raw SATA disk (matching the settings proven in Tanix testing).

### Driving the VM from the command line

- Screenshot: `"/Applications/VMware Fusion.app/Contents/Library/vmcli" build/vm/tanux-test.vmx MKS captureScreenshot out.png`
- Mouse and keyboard: `python3 scripts/vnc_input.py move X Y`, `click X Y`, `type "text"`, `key enter|down|up|tab|backspace|end` (uses VNC on port `5901`).

## Automatic builds (prepared, not active)

[`.github/workflows/build-iso.yml`](../.github/workflows/build-iso.yml) mirrors Tanix's workflow design: it runs on `ubuntu-24.04-arm` inside `quay.io/fedora/fedora:45`, runs `./scripts/check-kickstarts.sh` and `./scripts/build-iso.sh`, and uploads `tanux-latest-aarch64.iso` plus `.sha256` to Cloudflare R2.

Like Tanix, this workflow is **inactive and untested** while Tanux lives inside the incubator repository (GitHub Actions only runs workflows at the root of a repository). Once extracted to its own repo, configure `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY` secrets and the `R2_BUCKET` variable.
