# rig — TODO

A Rust CLI to spin GPU instances on Vast.ai up and down on demand. Personal tool for managing ephemeral GPU compute during a self-study curriculum.

---

## Done

- [x] `rig up` — find cheapest matching offer, create instance, poll until running, save state
- [x] `rig down` — destroy instance, clear state, show session cost
- [x] `rig status` — show running instance info and cost so far
- [x] `rig ssh` — SSH into running instance
- [x] Config loading from `~/.config/rig/config.toml` with env var override
- [x] State persistence at `~/.config/rig/state.json`
- [x] Stale state detection in `rig up` — verifies instance is live before blocking
- [x] Graceful handling of externally-destroyed instances in `rig down`
- [x] Sort offers by price, pick cheapest

---

## Todo

### 1. SSH key support
Pass SSH key to instance at creation time so `rig ssh` actually works.
- Add `ssh_key_id` to config (Vast.ai uses numeric key IDs)
- Pass it in the `create_instance` request body
- Figure out how to query the user's registered SSH keys from the Vast.ai API

### 2. File sync (`rig sync`)
Ergonomic way to send local code to the instance for CUDA execution.
- `rig sync` — rsync local directory to instance (e.g. `rsync -avz ./ root@<ip>:~/work/`)
- Needs IP and SSH port from state
- Should respect a `.rigignore` or just use `.gitignore`
- Consider: `rig run <script>` — sync + execute in one command

### 3. Config management
Config defaults are hardcoded in `config.rs`. Users need a proper way to set preferences.
- `rig config init` — write a default config file to `~/.config/rig/config.toml`
- `rig config show` — print current effective config (file + env var overrides)
- Document the config fields and valid values

### 4. DPH discrepancy
The price shown by `rig up` differs from the Vast.ai UI. Investigate whether we should use `dph_base` vs `dph_total` vs the `search.totalHour` field from the offer response.

### 5. Polish
- Remove debug logging from `get_instance`
- Clean up unused imports and warnings
- Add `--size` flag to `rig up` for quick GPU tier selection (e.g. `rig up --size rtx3090`)
- Timeout on the polling loop in `rig up` (currently loops forever if instance never starts)

---

## Stack
- Language: Rust (tokio, reqwest, clap v4, anyhow, serde)
- Provider: Vast.ai (switched from DigitalOcean — no GPU availability)
- Config: `~/.config/rig/config.toml`
- State: `~/.config/rig/state.json`
