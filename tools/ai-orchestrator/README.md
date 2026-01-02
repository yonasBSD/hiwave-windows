# AI Orchestrator (local-first)

This directory contains the **local-first multi-agent orchestration tooling** used to accelerate the Rust WebKit rewrite effort.

## Core concepts
- **WorkOrder**: a small, verifiable unit of work (1–3 days) with explicit acceptance gates.
- **VerificationReport**: the evidence bundle proving a WorkOrder is complete (build/test/stress logs + artifacts).
- **RoadmapIndex**: the shared index of WorkOrders (dependencies + status).

## Repository layout
- `tools/ai-orchestrator/aiorch.py`: CLI for initializing/validating WorkOrders and running local gates.
- `tools/ai-orchestrator/schemas/`: JSON Schemas for all orchestrator artifacts.
- `.ai/`: repo-local state and templates (tracked where appropriate).
  - `.ai/roadmap_index.json`: shared roadmap index (tracked)
  - `.ai/work_orders/`: WorkOrders (tracked)
  - `.ai/reports/`: Verification reports (tracked when desired)
  - `.ai/artifacts/`: run artifacts (gitignored)

## Quick start (manual)
From repo root:

```powershell
python tools/ai-orchestrator/aiorch.py validate-roadmap
```



## Artifact layout
All runtime outputs are stored under \\.ai\\artifacts\\. Each run writes a **manifest** at \\manifest.json\\ to make evidence references stable.

- CI: \\.ai/artifacts/<run_id>/<work_order_id>/\\ (stdout/stderr per gate + manifest)
- Swarm: \\.ai/artifacts/<run_id>/<work_order_id>/swarm/\\ (role JSON handoffs + policy report + manifest)
- Canary: \\.ai/artifacts/<run_id>/canary/\\ (build/run logs + manifest)
- Bisect: \\.ai/artifacts/<run_id>/bisect/\\ (bisect logs + manifest)

Reports are written to \\.ai/reports/\\ and always include relative paths into the artifact tree.

