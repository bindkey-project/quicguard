# lab

The QuicGuard test lab, managed as code with GitOps.

The lab grows in two stages:

- **v1 (current)**: a single Intel Mac, `qg-lab-1`. It is both the GitHub Actions self-hosted runner and the KVM host that runs the test VMs. Every test runs inside VMs, and the kernel module is never loaded on the host.
- **v2 (later)**: two Macs on a direct link, with a Raspberry Pi as the runner. Only v2 will test the module on bare metal.

## How it works

The `main` branch is the single source of truth. The Mac pulls its own configuration from it. Nobody pushes changes to it by hand.

```
  PR merged into main
          │
          ▼
       GitHub  ◀── git clone/pull ──  qg-gitops.timer (every 15 min, as root)
          │                                 │
          │                          ansible-pull → lab/ansible/site.yml
          │                                 │
          │                                 ▼
          └── jobs (runs-on: qg-lab) ──▶  qg-runner.service (user gh-runner)
                                           └─ builds, starts VMs, runs tests
```

- The Mac always opens the connections to GitHub. No inbound port is needed and no secret is stored in GitHub.
- Any manual change on the Mac to a file that a role manages is overwritten on the next run.

## Layout

```
lab/
└── ansible/
    ├── ansible.cfg
    ├── inventory.yml        # hosts (addresses live in host_vars/, ignored by Git)
    ├── site.yml             # the single playbook: applies every role to group "lab"
    └── roles/
        ├── base/            # dnf kernel exclusion, kernel.panic=10, SSH key-only, no dnf-makecache
        ├── libvirt/         # virtualization packages, modular libvirt daemons, test networks
        ├── gitops/          # ansible-pull service + timer
        └── runner/          # gh-runner user, GitHub Actions runner, systemd unit, CI lock
```

### Roles

| Role | What it manages |
|---|---|
| `base` | `excludepkgs=kernel*` in `dnf.conf`; `kernel.panic=10` (reboot 10 s after a panic); SSH password authentication disabled; `dnf-makecache.timer` disabled |
| `libvirt` | The `@virtualization` group and `python3-lxml`. It switches from the monolithic `libvirtd` to the modular daemons (`virtqemud`, `virtnetworkd`, `virtstoraged`) and defines the isolated test networks |
| `gitops` | `git`, `ansible`, `qg-gitops.service` (runs `ansible-pull` on `main`) and `qg-gitops.timer` (every 15 min) |
| `runner` | The `gh-runner` system user (groups `libvirt` and `kvm`, no sudo). It installs a pinned runner version with a checksum, registers it once, and manages `qg-runner.service` and the `/run/qg-lab` lock directory |

### Test networks

| libvirt network | Bridge | Use |
|---|---|---|
| `default` | `virbr0` | NAT to the Internet, for VM administration |
| `qg-net-a` | `qg-br-a` | Isolated: client ↔ router |
| `qg-net-b` | `qg-br-b` | Isolated: router ↔ server |

The isolated networks have no `<forward>` and no `<ip>`. They have no route outside, and the host has no address on them. Their MTU is 9000, so each VM can test with 1500 or 9000.

## Changing the lab

1. Change a role on a branch.
2. Test it from your machine before merging (see below).
3. Merge into `main`. The Mac applies the change within 15 minutes.

A broken role on `main` breaks the lab, so always test before merging.

### Running the playbook from your machine

Requirements: Ansible (`sudo dnf install ansible`) and SSH key access to the host.

The host address is kept out of Git. Create `lab/ansible/host_vars/qg-lab-1.yml`:

```yaml
ansible_host: <ip of the Mac>
```

Then, from `lab/ansible/`:

```bash
ansible lab -m ping                          # connectivity check
ansible-playbook site.yml --check --diff -K  # dry run
ansible-playbook site.yml --diff -K          # apply
```

A second run must report `changed=0`, which shows that the roles are idempotent.

### Operating GitOps on the Mac

```bash
systemctl list-timers qg-gitops.timer          # last and next run
sudo systemctl start qg-gitops.service         # reconcile now
journalctl -u qg-gitops.service -n 50          # output of the last run (look for PLAY RECAP)
```

## CI runner

- Labels: `self-hosted`, `Linux`, `X64`, `qg-lab`. Workflows target it with `runs-on: [self-hosted, qg-lab]`.
- One runner means one job at a time. Other jobs wait in the queue.
- `.github/workflows/lab-smoke.yml` checks the whole chain: runner identity, libvirt access and the lock.

### The CI lock

Running `ansible-pull` during a job would skew benchmarks and could restart the runner. A job that needs a quiet machine takes the lock:

```yaml
- name: Take lab lock
  run: touch /run/qg-lab/busy
# ...
- name: Release lab lock
  if: always()
  run: rm -f /run/qg-lab/busy
```

While `/run/qg-lab/busy` exists, `qg-gitops.service` skips its run (`ExecCondition`). `/run` is cleared at boot, so a crash cannot leave the lock behind.

### Security (public repository)

A self-hosted runner executes workflow code on our hardware, and anyone can fork a public repository.

- In *Settings > Actions > General*, require approval for all external contributors.
- Never trigger self-hosted jobs on `pull_request` from forks. Use `push` and `workflow_dispatch` only.
- The runner runs as `gh-runner`: no sudo and no `wheel`, only the `libvirt` and `kvm` groups. Its unit sets `NoNewPrivileges`, `ProtectSystem=full` and `PrivateTmp`.
- Workflows set `permissions: contents: read`.
- Never commit secrets (runner token, private keys) or host addresses.

## Setting up a new host from scratch

1. Install Fedora Server, enable SSH with a key, and create an admin user.
2. Add the host to `inventory.yml` and its address to `host_vars/<name>.yml`.
3. In GitHub, go to *Settings > Actions > Runners > New self-hosted runner* and copy only the token.
4. Bootstrap from your machine:
   ```bash
   read -rs RUNNER_TOKEN
   ansible-playbook site.yml --diff -K -e runner_token="$RUNNER_TOKEN"
   ```
   The token is only used when the runner is not registered yet. It is hidden with `no_log`, and `ansible-pull` never needs it.
5. Check that `systemctl status qg-runner` shows `Listening for Jobs` and that the runner is **Idle** in GitHub.

From then on, the host manages itself from `main`.

## Known pitfalls

| Symptom | Cause | Fix in the repo |
|---|---|---|
| `ansible-pull`: *Unable to advance to origin/main because local commits will be lost* | Shallow clone (`depth=1`) plus merge commits on `main` | `--clean` in `qg-gitops.service` |
| Runner: *No usable version of libssl was found* | Fedora 45 ships OpenSSL 4, but the runner's .NET needs `libssl.so.3` | `openssl3-libs` in the runner dependencies |
| `qg-runner.service`: `203/EXEC`, *Permission denied* | `bin/runsvc.sh` is not executable, and SELinux blocks systemd from executing files in `/var/lib` | `ExecStart=/usr/bin/bash …/bin/runsvc.sh` |
| `--check` fails on a `dnf` task | `python3-libdnf5` is missing until the first real run | Run once without `--check` |
| *Module invocation had junk after the JSON data* | Shows up only with `sudo` over SSH | Harmless, ignore it |

## Next steps

- Build a base VM image as code.
- Create the client, router and server topology on `qg-net-a` and `qg-net-b` from a CI job, then destroy it afterwards.
