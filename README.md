lazi 🦥
*The lazy pentester's automated Infrastructure-as-Code (IaC) tool.*

**Lazi** is a high-performance, declarative configuration engine written in Rust. It automates the complete lifecycle of Offensive Security virtual machines. By parsing a single YAML recipe, Lazi orchestrates local hypervisors (VirtualBox) to instantly clone, configure, and boot a headless Kali Linux VM, followed by establishing an SSH tunnel to inject custom toolchains and dotfiles.

Setup time is reduced from hours of manual configuration to roughly 3 minutes of zero-touch execution.

## Features
* **Zero-Touch Provisioning:** Directly puppets `VBoxManage` to clone base images, configure hardware (RAM/CPUs), force NAT port-forwarding, and boot headlessly.
* **Declarative YAML Manifests:** Define your exact engagement needs in a single file. Group tools by `apt`, `pipx`, or custom execution scripts.
* **Dotfile Injection:** Parses multi-line strings from the YAML and injects them directly into the VM via safe Bash Heredocs, automatically mapping ownership to the newly created user.

## Prerequisites
* **Host OS:** Linux (Tested on Ubuntu)
* **Hypervisor:** VirtualBox installed (`VBoxManage` must be in your PATH)
* **Rust:** `cargo` and `rustc` installed, along with `libssl-dev`
* **Golden Image:** A fully installed Kali Linux VirtualBox `.vdi` registered in VirtualBox under the exact name `kali-base`.

## Usage

1. Create a recipe file (e.g., `lazi-recipe.yaml`):

```yaml
name: "lazi-daily-driver"
vm:
  cpus: 4
  ram_mb: 8192

user:
  username: "pentester"
  shell: "/bin/zsh"

packages:
  apt:
    - nmap
    - ffuf
    - seclists
  pipx:
    - netexec

files:
  - path: "{{HOME}}/.tmux.conf"
    content: |
      set -g mouse on
      set -g default-terminal "screen-256color"
```

Deploy the environment:

```Bash
cargo run -- deploy lazi-recipe.yaml
```
