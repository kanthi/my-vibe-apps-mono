# Building Tanix on macOS

Tanix is a Linux system, so it cannot be built by macOS itself. The build runs inside a small NixOS machine in [OrbStack](https://orbstack.dev), and the files stay in this folder on the Mac.

## What you need

- An Apple Silicon Mac with OrbStack installed.
- About 25 GB of free disk for the builder's package store, more if you rebuild often (see "Free disk space").
- Internet access during the build.

Nix does not need to be installed on the Mac.

## One-time setup

```sh
./scripts/setup-builder.sh
```

This creates an OrbStack machine named `tanix-builder` running NixOS (arm64). OrbStack mounts your Mac home folder inside it at the same path, which is how the builder sees this folder.

To use a different machine name, set `TANIX_BUILDER` in your shell before running any script.

## Build the ISO

```sh
./scripts/build-iso.sh
```

The result is `~/Workspace/FusionVMs/tanix/tanix-<version>-aarch64-linux.iso` (set `TANIX_OUT` to change the folder), about 3 GB. The first build downloads several GB. Later builds reuse what is already there and mostly just repack the image, which takes a few minutes.

**Heat and speed.** Packing the image compresses about 3 GB and uses every core it is given. The scripts limit builds to 4 cores so the Mac stays usable and cooler. Set `TANIX_BUILD_CORES` to change that (for example `TANIX_BUILD_CORES=8 ./scripts/build-iso.sh` for a faster, hotter build).

What the script does:

1. Runs `nix build path:.#iso` inside the builder.
2. Copies the ISO into `~/Workspace/FusionVMs/tanix`. This copy is needed because Nix leaves the ISO in the builder's own store, which macOS cannot open.

**Keep big or changing files out of the app folder.** `path:.` makes Nix copy the whole app folder into its store on every build, ignored files and `build/` included. Until 2026-11-17 the ISO and the test VM lived in `build/`, so every build stored another copy of the previous 3 GB ISO (likely why the builder's store had reached 95 GB on 2026-11-16). And a file still being written, such as a build log, fails the build with "unexpected end-of-file". That is why ISOs and VMs now go to `~/Workspace/FusionVMs/tanix`. `build/` is still ignored by git, for small scratch files.

### Two details that matter

- **`path:.`** tells Nix to read this folder as it is on disk. Without it, Nix only sees files that git tracks, and new files are silently left out.
- **Flakes** are not switched on by default in the builder, so every `nix` command passes `--extra-experimental-features 'nix-command flakes'`. The scripts do this for you.

## Check the installer's choices

```sh
./scripts/check-desktops.sh            # all choices
./scripts/check-desktops.sh hyprland server
```

For each choice on the installer's "System" page, this puts together the same `configuration.nix` the installer would write and asks Nix to evaluate it. It finds renamed options and desktops that are broken in the pinned NixOS release. It takes about a minute per choice and does not build or boot anything. Run it after changing the installer or after `nix flake update`.

## Test the ISO in a VM

The builder cannot run the ISO: OrbStack machines share one kernel and are not full VMs. Use VMware Fusion, which runs the ISO at full speed with hardware virtualisation.

```sh
./scripts/fusion-vm.sh         # make a fresh test VM in ~/Workspace/FusionVMs/tanix/vm and boot the newest ISO
./scripts/fusion-vm.sh disk    # after installing: boot the installed system
./scripts/fusion-vm.sh stop    # power it off
```

The VM appears in VMware Fusion as "Tanix test" (open Fusion to see its window). It has 4 CPU cores, 4 GB of memory and a 30 GB disk.

The script encodes three things found by testing; they apply equally if you create the VM by hand:

- **Attach the ISO as a disk, not a CD.** As a CD, the live system got read errors partway through an install. As a disk it behaves like a USB stick, and installs complete.
- **Use bridged networking.** With Fusion's NAT the installer reported no internet on this Mac.
- **Use the vmxnet3 network card.** With e1000e the guest had no network.

An install needs the network, because the chosen system is downloaded during installation. A Server: Minimal install took about four minutes.

### Driving the VM from the command line

For checks without clicking by hand:

- Screenshot: `"/Applications/VMware Fusion.app/Contents/Library/vmcli" ~/Workspace/FusionVMs/tanix/vm/tanix-test.vmx MKS captureScreenshot out.png`
- Mouse and keyboard: `python3 scripts/vnc_input.py click 1007 669`, `... type hello`, `... key enter`. It talks to the VM's built-in remote display on port 5901, which `fusion-vm.sh` switches on.

### Do not test with emulation

An earlier attempt ran the ISO under QEMU inside the builder, without hardware acceleration. It works, but it holds 8 cores at 100% for as long as the VM is up and made this Mac overheat. Use Fusion.

## Free disk space

Every ISO build leaves a 3 GB image and its parts in the builder's store, and nothing removes them automatically. After two days of work the store had reached 95 GB. To clear everything not currently needed:

```sh
orb -m tanix-builder nix-collect-garbage
```

The next build then downloads its tools again. OrbStack returns the freed space to macOS gradually. To remove the builder entirely: `orb delete tanix-builder`.

## Updating NixOS

The NixOS revision is pinned in `flake.lock`. To move to a newer one:

```sh
orb -m tanix-builder bash -lc "cd '$PWD' && nix --extra-experimental-features 'nix-command flakes' flake update --flake path:."
./scripts/check-desktops.sh
./scripts/build-iso.sh
```

If the build stops at "applying patch", the installer changed upstream. See [installer.md](installer.md#regenerating-the-patch).

## Automatic builds (prepared, not active)

The plan, decided on 2026-11-16: GitHub Actions builds the ISO, and Cloudflare R2 hosts it. Only the latest build is kept, as one file, until Tanix settles.

- **Why GitHub for building:** free build time on public repositories, with arm64 runners.
- **Why R2 for the download:** a GitHub release file is limited to 2 GB and the ISO is about 3 GB. R2 takes it as one file and does not charge for downloads. One ISO fits easily in R2's free 10 GB.

`.github/workflows/build-iso.yml` is written for this but **has never run**, so expect to fix details on the first run. It does nothing while Tanix lives in the incubator repository, because GitHub only reads workflows at the root of a repository.

To switch it on:

1. Extract Tanix into its own GitHub repository (see the incubator's root `AGENTS.md`, "Lifecycle"). Public, for the free build time.
2. In Cloudflare: enable R2, create a bucket, turn on public access for it, and create an API token with write access to that bucket.
3. In the GitHub repository settings, add secrets `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID` and `R2_SECRET_ACCESS_KEY`, and a variable `R2_BUCKET`.
4. Run the workflow by hand from the Actions tab, or push a tag starting with `v`.

Each run overwrites `tanix-latest-aarch64.iso` and its `.sha256` file in the bucket.

The limits above (2 GB release files, 10 GB free R2 storage, free arm64 runners for public repositories) were stated from memory and not checked against the providers' current terms.

## x86_64

`flake.nix` also defines an x86_64 ISO (`nixosConfigurations.iso-x86_64`). It has never been built. On an Apple Silicon Mac it would have to be built under emulation, which is slow; a CI runner on x86_64 is the better place for it.
