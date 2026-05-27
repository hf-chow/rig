# Rig Roadmap: Multi-Provider GPU Aggregator

## Vision
Transform `rig` from a Vast.ai-only CLI into a GPU price aggregator that compares prices across multiple providers and automatically selects the cheapest option.

---

## Phase 1: Architecture Refactoring (Foundation)

### Goal
Create a provider-agnostic architecture that supports multiple GPU cloud providers.

### Tasks
- [ ] Create `GpuProvider` trait with standard interface
  - `list_offers()` - Query available GPU instances
  - `create_instance()` - Rent a GPU instance
  - `get_instance()` - Check instance status
  - `destroy_instance()` - Terminate instance
- [ ] Create normalized data structures
  - `NormalizedOffer` - Unified offer representation across providers
  - `NormalizedInstance` - Unified instance representation
  - `SearchCriteria` - Provider-agnostic search parameters
- [ ] Refactor `VastClient` to implement `GpuProvider` trait
- [ ] Update commands to use trait abstraction instead of concrete `VastClient`
- [ ] Update config to support multiple providers

**Estimated Time:** 1-2 days

---

## Phase 2: RunPod Integration

### Why RunPod First?
- GraphQL API (different from Vast.ai REST, good test of abstraction)
- Dual pricing model (spot + on-demand)
- Largest marketplace alternative to Vast.ai
- Well-documented API

### Provider Details
**API:** GraphQL at `https://api.runpod.io/graphql`  
**Auth:** Bearer token in header  
**Docs:** https://docs.runpod.io/reference/graphql-api

### Key Endpoints
```graphql
# List available GPUs
query { gpuTypes { id, displayName, memoryInGb } }

# Create spot instance
mutation { podRentInterruptable(input: {...}) { id, imageName } }

# Check status
query { pod(input: {podId: "..."}) { id, runtime, desiredStatus } }

# Destroy
mutation { podTerminate(input: {podId: "..."}) { id } }
```

### Tasks
- [ ] Implement `RunPodClient` with GraphQL queries
- [ ] Implement `GpuProvider` trait for RunPod
- [ ] Map RunPod GPU types to normalized offers
- [ ] Handle spot vs on-demand instance types
- [ ] Add RunPod config section (API key, prefer spot/on-demand)
- [ ] Test full lifecycle: up → status → ssh → down

**Estimated Time:** 2-3 days

---

## Phase 3: Lambda Labs Integration

### Why Lambda Labs?
- Simplest REST API (clean reference implementation)
- Fixed pricing (no bidding complexity)
- Popular among ML practitioners

### Provider Details
**API:** REST at `https://cloud.lambdalabs.com/api/v1/`  
**Auth:** Bearer token  
**Docs:** https://cloud.lambdalabs.com/api/v1/docs

### Key Endpoints
```bash
GET  /instance-types              # List available GPUs
POST /instance-operations/launch  # Create instance
GET  /instances/{id}              # Check status
POST /instance-operations/terminate # Destroy
```

### Tasks
- [ ] Implement `LambdaClient` with REST endpoints
- [ ] Implement `GpuProvider` trait for Lambda Labs
- [ ] Map Lambda instance types to normalized offers
- [ ] Handle region selection
- [ ] Add Lambda config section (API key, preferred regions)
- [ ] Test full lifecycle

**Estimated Time:** 1-2 days

---

## Phase 4: Paperspace Integration

### Why Paperspace?
- Managed platform features (GUI access, notebooks)
- Hybrid pricing model (different from spot/fixed)
- Enterprise-grade reliability

### Provider Details
**API:** REST at `https://api.paperspace.io/`  
**Auth:** `x-api-key` header  
**Docs:** https://docs.paperspace.com/core/api-reference

### Key Endpoints
```bash
GET  /machines/getMachineTypes           # List GPUs
POST /machines/createSingleMachinePublic # Create
GET  /machines/getMachineById/{id}       # Status
POST /machines/destroyMachine            # Destroy
```

### Tasks
- [ ] Implement `PaperspaceClient` with REST endpoints
- [ ] Implement `GpuProvider` trait for Paperspace
- [ ] Map Paperspace machine types to normalized offers
- [ ] Handle subscription vs pay-as-you-go pricing
- [ ] Add Paperspace config section
- [ ] Test full lifecycle

**Estimated Time:** 1-2 days

---

## Phase 5: Price Comparison & Auto-Selection

### Goal
Query all providers simultaneously and automatically select the cheapest option.

### New Commands
```bash
rig compare    # Show price comparison across all providers
rig up --auto  # Auto-select cheapest provider (default behavior)
rig up --provider runpod  # Force specific provider
```

### Tasks
- [ ] Implement parallel provider queries
- [ ] Create comparison table display (provider, GPU, price, location)
- [ ] Add price sorting logic across providers
- [ ] Update `rig up` to query all providers by default
- [ ] Add `--provider` flag to override auto-selection
- [ ] Store selected provider in state.json
- [ ] Update `rig status` to show provider info
- [ ] Add provider-specific pricing notes (spot risk, etc.)

**Estimated Time:** 2-3 days

---

## Phase 6: Enhanced Features

### Nice-to-Have Improvements
- [ ] Cache provider offers for N minutes to reduce API calls
- [ ] Add `--prefer-reliability` flag (favor on-demand over spot)
- [ ] Support for multi-GPU instances
- [ ] Provider health/availability tracking
- [ ] Cost estimation before renting
- [ ] Historical price tracking
- [ ] Provider reputation scoring

**Estimated Time:** Ongoing

---

## Configuration Format (Future)

```toml
# ~/.config/rig/config.toml

# Global preferences
min_vram_gb = 24
max_price_per_hour = 0.50
image = "pytorch/pytorch:2.3.0-cuda12.1-cudnn8-runtime"
prefer_reliability = false  # true = avoid spot instances

# Provider configs
[providers.vastai]
enabled = true
api_key = "..."  # or use VASTAI_API_KEY env var
ssh_key_id = [12345]

[providers.runpod]
enabled = true
api_key = "..."  # or use RUNPOD_API_KEY env var
prefer_spot = true

[providers.lambda]
enabled = true
api_key = "..."  # or use LAMBDA_API_KEY env var
preferred_regions = ["us-west-1", "us-east-1"]

[providers.paperspace]
enabled = false  # disable expensive providers
api_key = "..."  # or use PAPERSPACE_API_KEY env var
```

---

## Research Summary

### API Comparison

| Provider | API Type | Auth Header | Pricing Model | Availability |
|----------|----------|-------------|---------------|--------------|
| Vast.ai | REST | `api_key` param | Spot marketplace | High |
| RunPod | GraphQL | `Authorization: Bearer` | Spot + On-demand | High |
| Lambda Labs | REST | `Authorization: Bearer` | Fixed hourly | Medium (often sold out) |
| Paperspace | REST | `x-api-key` | Subscription + PAYG | Medium |

### Implementation Order Rationale
1. **RunPod** - Tests GraphQL abstraction, dual pricing model
2. **Lambda Labs** - Simplest API, validates REST abstraction
3. **Paperspace** - Different auth method, managed platform features

---

## Success Metrics
- [ ] Support 4 GPU providers (Vast.ai, RunPod, Lambda, Paperspace)
- [ ] Price comparison across all providers in <2 seconds
- [ ] Automatic selection of cheapest option
- [ ] Full instance lifecycle working for each provider
- [ ] Single config file manages all providers
- [ ] User can override provider selection

---

## Out of Scope (For Now)
- Major cloud providers (AWS/GCP/Azure) - too expensive, complex APIs
- Tensordock, Genesis Cloud, Jarvislabs - wait until Tier 1 stable
- Multi-instance management
- GPU performance benchmarking
- Automatic failover between providers
