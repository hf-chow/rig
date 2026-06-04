# rig

A CLI to spin GPU instances on [Vast.ai](https://vast.ai) up and down on demand.

```
rig up      # rent the cheapest matching GPU
rig ssh     # SSH into the running instance
rig status  # show uptime and session cost
rig down    # destroy instance and print final cost
```

## Install

```bash
cargo install --path .
```

Requires Rust. Install via [rustup](https://rustup.rs) if needed.

## Setup

**1. Get your Vast.ai API key**

Go to [vast.ai/console/account](https://vast.ai/console/account) and copy your API key.

**2. Set the environment variable**

```bash
echo 'export VASTAI_API_KEY=your_key_here' >> ~/.bashrc
source ~/.bashrc
```

**3. Add your SSH key to Vast.ai**

Go to the Vast.ai console → Account → SSH Keys, upload your public key, and note the numeric key ID.

**4. Create a config file**

```bash
mkdir -p ~/.config/rig
```

Create `~/.config/rig/config.toml`:

```toml
# Global search preferences (shared across providers)
min_vram_gb = 24              # minimum VRAM in GB
max_price_per_hour = 0.50     # budget cap in USD/hr
image = "pytorch/pytorch:2.3.0-cuda12.1-cudnn8-runtime"

# Per-provider settings
[providers.vastai]
ssh_key_id = [12345]          # your Vast.ai SSH key ID (required for rig ssh)
api_key = "..."               # optional; prefer the VASTAI_API_KEY env var
```

## Config reference

Global preferences (top level), shared by all providers:

| Field               | Default                                        | Description                  |
|---------------------|------------------------------------------------|------------------------------|
| `min_vram_gb`       | `8`                                            | Minimum GPU VRAM in GB       |
| `max_price_per_hour`| `0.05`                                         | Max all-in price in USD/hr   |
| `image`             | `pytorch/pytorch:2.3.0-cuda12.1-cudnn8-runtime`| Docker image for the instance|
| `disk_gb`           | `10`                                           | Disk size in GB              |

Vast.ai provider settings, under `[providers.vastai]`:

| Field         | Default | Description                              |
|---------------|---------|------------------------------------------|
| `ssh_key_id`  | —       | Vast.ai SSH key IDs (array of ints)      |
| `api_key`     | —       | API key (prefer the `VASTAI_API_KEY` env var) |

The `VASTAI_API_KEY` environment variable takes precedence over `api_key` in the config file.

## Usage

```
# Spin up a GPU instance
rig up

# Check running instance info and cost
rig status

# SSH in
rig ssh

# Tear it down
rig down
```

`rig up` picks the cheapest available offer matching your config, rents it, and polls until ready. State is saved to `~/.config/rig/state.json`.

## Stack

- Rust (tokio, reqwest, clap v4, serde, anyhow)
- [Vast.ai](https://vast.ai) API
