# Issue #66 status — 2026-09-16

## Source path (A0–A4) — PASS

- Root cause: FTDI `latency_timer` 16 → 1 (not Max_Info_Frames alone).
- RP spans: `/api/metrics/rp-spans` + unit tests.
- 30m Workbench soak evidence under `docs/evidence/ISSUE66_*` (A4 PASS; p95 ~38–40 ms with latency=1, MIF=2).
- PR #67 merged into #65 (`feat/lab-persist-systemd-ansible`).

## Lab leave-running

- workerpi1 `192.168.204.59` router BIP↔MS/TP DNET **2001** (never 2000).
- workerpi2 `192.168.204.60` mini instance **123102** MAC2.
- FTDI latency_timer=1 confirmed; Pis left up for Workbench.

## Phase B — Pi image

- `rpi3_64` lab commissioning example present.
- **Flash blocked on Ben confirmation of `/dev/sdX`.** Source path remains the live lab until then.

## Phase C — x86 live ISO

- Buildroot produces `rootfs.iso9660` (symlinked `rootfs.iso`).
- GRUB fix: `BR2_TARGET_GRUB2_BOOT_PARTITION=cd` + `iso9660` module (was dropping to `grub>` with `hd0,msdos1`).
- CI (#68 on master): ext2 `qemu-smoke` **hard** gate; `-cdrom` smoke **continue-on-error Soft** until INITRD live reaches healthz.
- Master `build-os` green after Soft gate (`c39aa5e`).
- Disposable VMware boot still operator-gated after CI `-cdrom` green.

## Open-FDD smoke

- BIP via `.59` → DNET 2001 / MAC2 only (never plant DNET 2000).
