# Lab Ansible — dual mini + bensbench router

Bensbench (`192.168.204.11`) runs the DIY **router** (source `routerd`).
Both Raspberry Pis run **mstp-mini-device** fixtures on the shared RS-485 trunk.

## Topology (locked)

| Host | Role |
|---|---|
| bensbench `192.168.204.11` | `diy-bacnet-router --route-enable` MAC1 DNET **2001** (Waveshare C) |
| bacpypes oracle | Different BIP IP than `.11` — prefer `.12` alias on bensbench, or run bacpypes on a Pi Ethernet address |
| workerpi2 `192.168.204.60` | mini MAC **2** / instance **123102** |
| workerpi1 `192.168.204.59` | mini MAC **3** / instance **123103** |
| FEC | MAC **7** / **5007** — **on trunk only at 38400**; disconnect before other bauds |

**Never plant DNET 2000.** No FEC writes.

## Deploy dual minis

```bash
cd /path/to/diy-bacnet-router
ansible-playbook -i ansible/inventory/lab.yml ansible/playbooks/dual_mini_lab.yml
```

## Change baud (FEC gate)

```bash
# FEC may stay connected:
ansible-playbook -i ansible/inventory/lab.yml ansible/playbooks/set_baud.yml -e dbr_baud=38400

# After human disconnects FEC from the trunk:
ansible-playbook -i ansible/inventory/lab.yml ansible/playbooks/set_baud.yml \
  -e dbr_baud=9600 -e fec_disconnected=true
```

Then set matching `[mstp] baud` on bensbench and restart the router.

## Routed oracle

```bash
python scripts/lab_routed_rp_oracle.py \
  --address 192.168.204.59/24 --router 192.168.204.11 --include-fec
```

## Stop minis

```bash
ssh ben@192.168.204.59 'sudo systemctl stop mstp-mini-device'
ssh ben@192.168.204.60 'sudo systemctl stop mstp-mini-device'
```
