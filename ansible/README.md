# Lab Ansible — two-Pi BIP↔MS/TP

Deploys **persistent** systemd units so Workbench on another PC can discover
remote BACnet network **2001** / device **123102** through workerpi1.

## Topology (locked)

| Host | Role |
|---|---|
| workerpi1 `192.168.204.59` | `diy-bacnet-router --route-enable --qualify-secs 0` |
| workerpi2 `192.168.204.60` | Vibe13 `mstp-mini-device` MAC 2 / instance 123102 |

## Prerequisites

- SSH key access as `ben` with passwordless `sudo`
- Release binaries already built (or pass `-e rebuild_router=true -e rebuild_mini=true`)
- Isolated RS-485 trunk wired (no BASRT on this bus)

## Deploy

```bash
cd /path/to/diy-bacnet-router
ansible-playbook -i ansible/inventory/lab.yml ansible/playbooks/two_pi_lab.yml
```

Rebuild on deploy:

```bash
ansible-playbook -i ansible/inventory/lab.yml ansible/playbooks/two_pi_lab.yml \
  -e rebuild_router=true -e rebuild_mini=true
```

## Workbench

1. Discover → **Remote network 2001** (or Global 65535 fallback)
2. Instance bounds **123102** (unbounded Who-Is also works)
3. Map device; poll `Object_Name` and `analogInput:1 Present_Value`

## Management UI

Bound to `127.0.0.1:8080` on workerpi1. From Windows:

```powershell
ssh -N -L 18080:127.0.0.1:8080 ben@192.168.204.59
```

Open `http://127.0.0.1:18080`.

## Stop lab

```bash
ssh ben@192.168.204.59 'sudo systemctl stop diy-bacnet-router'
ssh ben@192.168.204.60 'sudo systemctl stop mstp-mini-device'
```
