#!/usr/bin/env python3
"""Install Windows SSH public key on the Ubuntu lab guest using password from config/vm.env."""
from __future__ import annotations

import sys
from pathlib import Path

try:
    import paramiko
except ImportError:
    print("paramiko required: pip install paramiko", file=sys.stderr)
    sys.exit(2)


def load_vm_env(repo_root: Path) -> dict[str, str]:
    env_path = repo_root / "config" / "vm.env"
    if not env_path.is_file():
        print(f"missing {env_path} — copy config/vm.env.example to config/vm.env", file=sys.stderr)
        sys.exit(2)
    values: dict[str, str] = {}
    for line in env_path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, _, val = line.partition("=")
        values[key.strip()] = val.strip()
    return values


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    cfg = load_vm_env(repo_root)
    user = cfg.get("VM_SSH_USER", "ben")
    host = cfg.get("VM_SSH_HOST", "127.0.0.1")
    port = int(cfg.get("VM_SSH_PORT", "2222"))
    password = cfg.get("VM_SSH_PASSWORD", "")
    if not password:
        print("Set VM_SSH_PASSWORD in config/vm.env", file=sys.stderr)
        return 2

    pub_path = next(
        (p for p in (Path.home() / ".ssh" / "id_rsa.pub", Path.home() / ".ssh" / "id_ed25519.pub") if p.is_file()),
        None,
    )
    if pub_path is None:
        print("No SSH public key in ~/.ssh/id_rsa.pub or id_ed25519.pub", file=sys.stderr)
        return 2

    pubkey = pub_path.read_text(encoding="utf-8").strip()
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    print(f"Connecting to {user}@{host}:{port} …")
    client.connect(
        hostname=host,
        port=port,
        username=user,
        password=password,
        look_for_keys=False,
        allow_agent=False,
        timeout=15,
    )

    sftp = client.open_sftp()
    try:
        sftp.stat(".ssh")
    except OSError:
        client.exec_command("mkdir -p .ssh && chmod 700 .ssh")[1].read()

    auth_path = ".ssh/authorized_keys"
    try:
        with sftp.open(auth_path, "r") as handle:
            existing = handle.read().decode("utf-8", errors="replace")
    except OSError:
        existing = ""

    if pubkey not in existing.splitlines():
        with sftp.open(auth_path, "w") as handle:
            merged = (existing.rstrip("\n") + "\n" + pubkey + "\n") if existing.strip() else (pubkey + "\n")
            handle.write(merged)
    client.exec_command("chmod 600 .ssh/authorized_keys")[1].read()
    sftp.close()
    client.close()

    print("KEY_INSTALLED")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
