---
name: AI Orchestrated Rewrite
overview: Set up a local-first multi-agent AI system that can autonomously plan, implement, test, and integrate the Rust WebKit rewrite work via PRs, with policy gates, CI-like local validation, canary rollouts, and automatic rollback on regressions.
todos:
  - id: define-agent-io
    content: Define machine-readable WorkOrder + VerificationReport schemas and a single shared roadmap index the swarm reads/writes.
    status: completed
  - id: local-ci-emulator
    content: Create a local CI emulator command that runs build/test/stress and outputs a signed merge report artifact.
    status: completed
  - id: agent-roles-runtime
    content: Implement the coordinator/architect/implementer/verifier agent runtime with tool access policies and structured handoffs.
    status: in_progress
  - id: repo-integration-bot
    content: Add PR automation locally (branch creation, commits, PR creation if remote exists, otherwise patch queues), with auto-merge policy enforcement.
    status: pending
  - id: canary-runner
    content: Implement a local canary runner that builds HiWave + runs scripted smoke flows (resize drag, multi-view, navigation) and publishes health signals.
    status: pending
  - id: auto-revert-bisect
    content: Implement automatic revert and local bisect workflow triggered by canary failures, generating new WorkOrders for fixes.
    status: pending
  - id: artifact-store
    content: Add a local artifact store layout for logs, traces, screenshots, and verification reports; ensure PRs reference artifacts deterministically.
    status: pending
  - id: guardrails
    content: "Add guardrails: file allowlists, dangerous-operation approvals, dependency allowlists, and mandatory evidence gates before merge."
    status: pending
  - id: test-first-policy
    content: Adopt a policy that every resolved limitation/regression adds a dedicated test (stress + deterministic reftest where possible).
    status: pending
  - id: throughput-metrics
    content: Instrument the orchestration system to track throughput, failure rate, and mean time to recovery; review weekly to tune prompts and gates.
    status: pending
---

# Multi-AI Orchestration Plan for Rust WebKit Rewrite (Local-first, Auto-merge + Canary)

## Objectives
- **Primary**: Use current AI agent tech to execute the work in [`rust_webkit_rewrite_05d3f01d.plan.md`](c:\Users\petec\.cursor\plans\rust_webkit_rewrite_05d3f01d.plan.md) in parallel, with high throughput and low regression rate.
- **Constraints**:
  - **Execution**: local-only (your Windows workstation).
  - **Autonomy**: agents can **open PRs, run validations, auto-merge**, and perform **canary rollout + auto-revert** based on health signals.

## Operating model (how the swarm works day-to-day)
### Work decomposition
- Convert each engineering milestone into **small, verifiable “work orders”** (1–3 days each) with:
  - clear acceptance tests
  - explicit files/modules touched
  - deterministic reproduction scripts
  - rollback plan

### Branching + integration strategy
- Use **trunk-based with short-lived branches**:
  - `main` = always releasable
  - `agent/<task>/<shortid>` = agent work branches
  - auto-merge requires all gates green

### Auto-merge and canary policy (local-first)
Since you’re local-only, emulate CI with a **local gate runner**.
- **Merge gates** (must pass):
  - build (debug + release where relevant)
  - unit tests
  - integration tests (HiWave harness + resize/multi-view stress)
  - formatting + clippy
  - security checks (dependency allowlist, optional)
- **Canary**: every merge to `main` triggers:
  - packaging a local build
  - running a scripted “smoke run” (navigate + sidebar drag + multi-view)
  - logging health metrics
- **Auto-revert**: if canary fails, swarm generates a revert PR and bisects.

## Agent swarm design
### Roles (specialized agents, run in parallel)
- **Coordinator** (1): owns global roadmap, assigns tasks, resolves conflicts, enforces policies.
- **Architect** (1): maintains engine/module boundaries, API contracts, and migration sequencing.
- **Implementers** (N=3–8): write code for a single work order end-to-end.
- **Test/Verification Agent** (1–2): writes/extends tests, stress harnesses, reftests, invariants.
- **Build/Tooling Agent** (1): improves build speed, caching, reproducibility, log capture.
- **Bug Triage Agent** (1): monitors failures, clusters regressions, proposes minimal fixes.
- **Doc/Spec Agent** (1): keeps design docs, ADRs, and interface specs in sync.

### Responsibilities and interfaces
Each agent communicates only through:
- **Work Orders** (structured JSON/YAML)
- **Design Contracts** (Rust traits, protocol schemas)
- **PR artifacts** (diff + test evidence + perf evidence)

## Tooling stack (local-first)
### Orchestrator framework
Pick one of these patterns (implement incrementally):
- **LangGraph** (Python) for graph-based workflows and tool calling.
- **AutoGen** for multi-agent chat + task routing.
- **Custom coordinator** (Rust or Python) if you prefer minimal deps.

Recommended for “clear/neat convergence”: **LangGraph** with explicit state machines.

### Shared artifacts (the glue that makes agents converge)
- **Single source of truth**: a machine-readable `RoadmapIndex`:
  - work orders
  - dependencies
  - acceptance criteria
  - owners
- **Decision log (ADRs)** for irreversible choices.
- **Interface spec** for HiWave embedder API (`IWebView` alignment) and the new engine API.

### Local “CI emulator”
- A single command the Coordinator invokes:
  - builds
  - runs tests
  - runs stress harnesses
  - generates a **merge report** artifact

## Workflow blueprint (end-to-end)

### 1) Intake → Work Order creation
- Coordinator reads the current roadmap item (e.g., `rustkit-compositor`).
- Architect produces an interface contract (traits + structs + invariants).
- Coordinator emits `WorkOrder` with:
  - goal
  - exact acceptance tests
  - performance budget
  - affected modules

### 2) Parallel implementation
- Implementer agent(s) each take one sub-slice:
  - example for compositor: `SurfaceState`, `ResizePath`, `MultiViewRegistry`.
- Test agent simultaneously creates:
  - resize torture tests
  - pixel-diff or checksum-based reftests

### 3) Local validation gates
- Build agent ensures deterministic builds and speeds up iteration.
- Verification agent runs:
  - unit + integration + stress
  - produces a `VerificationReport` (logs + metrics + pass/fail)

### 4) PR creation and review automation
- Implementer opens PR with:
  - structured summary
  - evidence links (local artifacts path)
  - risk assessment
  - rollback instructions
- Coordinator agent performs:
  - static review checks
  - conflict resolution
  - required approvals (policy-based)

### 5) Auto-merge → Canary → Auto-revert
- Coordinator merges when gates pass.
- Canary runner executes smoke scripts.
- If canary fails:
  - auto-revert
  - bisect automation
  - open bug work order

## Concurrency without chaos (critical)
### Preventing merge conflicts
- Enforce strict module ownership per work order.
- Use “API-first” work orders:
  - land scaffolding + interfaces first
  - then fill implementations

### Preventing architectural drift
- Architect agent owns:
  - module map
  - dependency rules
  - forbidden imports
- Add automated checks:
  - crate boundary enforcement
  - dependency graph linting

### Preventing hallucinated/incorrect code
- Hard requirement: **no PR merges without executable proof**.
- Require “repro scripts” for every bug fix.
- Maintain a regression test for each resolved WinCairo limitation.

## AI acceleration tactics specific to this project
### Use WebKit as a “behavior oracle” without copying its code
- Agents generate targeted tests derived from:
  - LayoutTests behavior
  - minimal reproduction cases for WinCairo bugs

### Create “golden traces” early
- Record:
  - navigation event sequences
  - resize sequences
  - compositing frame timing
- Agents must match golden traces as acceptance.

### Invest early in debug visibility
- Mandatory from week 1:
  - structured logging categories
  - frame debug HUD
  - event timeline capture

## Security and safety guardrails (autonomous system)
- Secrets never exposed to agent prompts.
- Local filesystem allowlist for tool access.
- “Dangerous operations” require Coordinator confirmation:
  - deleting large directories
  - rebasing main
  - rewriting history

## Metrics (what we track to know if the swarm is working)
- Lead time per work order
- Merge success rate
- Canary failure rate
- Mean time to auto-revert
- Test coverage growth for limitation regressions

## Immediate setup checklist (first 2 weeks)
- Implement WorkOrder schema + storage.
- Implement LocalCI emulator command.
- Stand up 5 core agents: Coordinator, Architect, 2 Implementers, 1 Verification.
- Add first 10 work orders covering:
  - `rustkit-viewhost`
  - `rustkit-compositor`
  - resize/multi-view stress harness

## Implementation todos