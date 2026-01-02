"""
aiorch.py - local-first AI orchestration utilities.

Design goals:
- Zero third-party dependencies (stdlib only) so it runs on a fresh Windows box.
- Schemas live in tools/ai-orchestrator/schemas/ as JSON Schema, but validation
  here is intentionally lightweight and pragmatic.

This tool intentionally separates:
- "Planning" (swarm run) which may call external LLM APIs
- "Proof" (ci run) which executes deterministic local gates and emits evidence
- "Integration" (repo *) which handles branches/patches and enforces merge policy
"""

from __future__ import annotations

import argparse
import concurrent.futures
import json
import os
import pathlib
import re
import secrets
import subprocess
import sys
import urllib.error
import urllib.request
from datetime import datetime, timezone
from hashlib import sha256
from typing import Any, Dict, List, Optional


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
AI_DIR = REPO_ROOT / ".ai"
ROLES_DIR = pathlib.Path(__file__).resolve().parent / "roles"


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def read_json(path: pathlib.Path) -> Dict[str, Any]:
    # PowerShell commonly writes UTF-8 with BOM; utf-8-sig accepts both.
    with path.open("r", encoding="utf-8-sig") as f:
        return json.load(f)


def read_text(path: pathlib.Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def write_text(path: pathlib.Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", errors="replace")


def write_json(path: pathlib.Path, obj: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=2), encoding="utf-8")


def die(msg: str) -> None:
    print(f"[aiorch] ERROR: {msg}", file=sys.stderr)
    raise SystemExit(2)


def ensure_dirs() -> None:
    (AI_DIR / "work_orders").mkdir(parents=True, exist_ok=True)
    (AI_DIR / "reports").mkdir(parents=True, exist_ok=True)
    (AI_DIR / "artifacts").mkdir(parents=True, exist_ok=True)
    (AI_DIR / "cache").mkdir(parents=True, exist_ok=True)


def _require(obj: Dict[str, Any], key: str, expected_type: Any) -> Any:
    if key not in obj:
        die(f"Missing required key '{key}'")
    val = obj[key]
    if expected_type is not Any and not isinstance(val, expected_type):
        die(f"Key '{key}' must be {expected_type}, got {type(val)}")
    return val


def _require_enum(obj: Dict[str, Any], key: str, allowed: List[str]) -> str:
    val = _require(obj, key, str)
    if val not in allowed:
        die(f"Key '{key}' must be one of {allowed}, got '{val}'")
    return val


def validate_roadmap_index(doc: Dict[str, Any]) -> None:
    _require(doc, "schema_version", str)
    _require(doc, "created_at", str)
    _require(doc, "updated_at", str)
    work_orders = _require(doc, "work_orders", list)

    ids: set[str] = set()
    for i, wo in enumerate(work_orders):
        if not isinstance(wo, dict):
            die(f"work_orders[{i}] must be an object")
        wo_id = _require(wo, "id", str)
        if wo_id in ids:
            die(f"Duplicate work order id: {wo_id}")
        ids.add(wo_id)
        _require(wo, "title", str)
        _require_enum(wo, "status", ["pending", "in_progress", "completed", "blocked", "cancelled"])
        deps = wo.get("depends_on", [])
        if deps is None:
            deps = []
        if not isinstance(deps, list) or any(not isinstance(d, str) for d in deps):
            die(f"work_orders[{i}].depends_on must be a list of strings")

    for wo in work_orders:
        for dep in wo.get("depends_on", []) or []:
            if dep not in ids:
                die(f"Work order '{wo['id']}' depends_on unknown id '{dep}'")


def work_order_path(work_order_id: str) -> pathlib.Path:
    return AI_DIR / "work_orders" / f"{work_order_id}.json"


def load_work_order(work_order_id: str) -> Dict[str, Any]:
    path = work_order_path(work_order_id)
    if not path.exists():
        die(f"WorkOrder not found: {path}")
    return read_json(path)


def make_run_id() -> str:
    return f"{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}_{secrets.token_hex(4)}"


# =========================
# Local CI emulator
# =========================


def run_gate(*, gate_id: str, cmd: str, timeout_seconds: int, artifact_dir: pathlib.Path) -> Dict[str, Any]:
    started = datetime.now(timezone.utc)
    out_path = artifact_dir / f"{gate_id}.stdout.txt"
    err_path = artifact_dir / f"{gate_id}.stderr.txt"

    try:
        proc = subprocess.run(
            cmd,
            cwd=str(REPO_ROOT),
            shell=True,
            capture_output=True,
            text=True,
            timeout=timeout_seconds,
        )
        write_text(out_path, proc.stdout or "")
        write_text(err_path, proc.stderr or "")
        exit_code = int(proc.returncode)
    except subprocess.TimeoutExpired as e:
        # Mark timeouts clearly; 124 mirrors common unix conventions.
        write_text(out_path, e.stdout if isinstance(e.stdout, str) else "")
        write_text(err_path, e.stderr if isinstance(e.stderr, str) else "")
        exit_code = 124
    except Exception as e:
        write_text(out_path, "")
        write_text(err_path, f"Exception while running gate '{gate_id}': {e}\n")
        exit_code = 125

    finished = datetime.now(timezone.utc)
    duration_ms = int((finished - started).total_seconds() * 1000)

    return {
        "id": gate_id,
        "cmd": cmd,
        "exit_code": exit_code,
        "duration_ms": duration_ms,
        "stdout_path": str(out_path.relative_to(REPO_ROOT)).replace("\\", "/"),
        "stderr_path": str(err_path.relative_to(REPO_ROOT)).replace("\\", "/"),
    }


def cmd_validate_roadmap(_: argparse.Namespace) -> None:
    ensure_dirs()
    roadmap_path = AI_DIR / "roadmap_index.json"
    if not roadmap_path.exists():
        die(f"Missing {roadmap_path}. Run init (or create it) first.")
    doc = read_json(roadmap_path)
    validate_roadmap_index(doc)
    print(f"[aiorch] OK: {roadmap_path} is structurally valid.")


def cmd_init(_: argparse.Namespace) -> None:
    ensure_dirs()
    roadmap_path = AI_DIR / "roadmap_index.json"
    if roadmap_path.exists():
        print(f"[aiorch] {roadmap_path} already exists; leaving as-is.")
        return

    doc = {
        "schema_version": "roadmap_index.v1",
        "created_at": utc_now_iso(),
        "updated_at": utc_now_iso(),
        "work_orders": [],
    }
    roadmap_path.write_text(json.dumps(doc, indent=2), encoding="utf-8")
    print(f"[aiorch] Created {roadmap_path}")


def cmd_ci_run(args: argparse.Namespace) -> None:
    ensure_dirs()

    wo = load_work_order(args.work_order)
    acceptance = _require(wo, "acceptance", dict)
    gates = _require(acceptance, "gates", list)
    if not gates:
        die(f"WorkOrder '{args.work_order}' has no acceptance.gates")

    run_id = make_run_id()
    artifact_dir = AI_DIR / "artifacts" / run_id / args.work_order
    artifact_dir.mkdir(parents=True, exist_ok=True)

    write_json(artifact_dir / 'manifest.json', {
        'schema_version': 'artifact_manifest.v1',
        'kind': 'ci',
        'run_id': run_id,
        'work_order_id': args.work_order,
        'created_at': utc_now_iso(),
    })

    started_at = utc_now_iso()
    gate_results: List[Dict[str, Any]] = []

    for gate in gates:
        if not isinstance(gate, dict):
            die("acceptance.gates entries must be objects")
        gate_id = _require(gate, "id", str)
        cmd = _require(gate, "cmd", str)
        timeout = int(gate.get("timeout_seconds", 1800))

        gate_results.append(run_gate(gate_id=gate_id, cmd=cmd, timeout_seconds=timeout, artifact_dir=artifact_dir))

        if args.fail_fast and gate_results[-1]["exit_code"] != 0:
            break

    finished_at = utc_now_iso()
    overall = "pass" if all(g["exit_code"] == 0 for g in gate_results) else "fail"

    report: Dict[str, Any] = {
        "schema_version": "verification_report.v1",
        "work_order_id": args.work_order,
        "status": overall,
        "started_at": started_at,
        "finished_at": finished_at,
        "gates": gate_results,
        "artifacts": {
            "artifact_dir": str(artifact_dir.relative_to(REPO_ROOT)).replace("\\", "/"),
            "logs": [g["stdout_path"] for g in gate_results] + [g["stderr_path"] for g in gate_results],
            "screenshots": [],
            "traces": [],
        },
    }

    report_json = json.dumps(report, indent=2)
    report_hash = sha256(report_json.encode("utf-8")).hexdigest()
    signed_report = {**report, "signature": {"type": "sha256", "value": report_hash}}

    report_path = AI_DIR / "reports" / f"{run_id}_{args.work_order}.verification.json"
    write_json(report_path, signed_report)

    print(f"[aiorch] CI run complete: status={overall}")
    print(f"[aiorch] VerificationReport: {report_path}")
    print(f"[aiorch] Artifact dir: {artifact_dir}")
    print(f"[aiorch] Signature (sha256): {report_hash}")
    emit_metric({
        "event_type": "ci_run",
        "work_order_id": args.work_order,
        "run_id": run_id,
        "status": overall,
        "duration_ms": sum(g["duration_ms"] for g in gate_results),
    })
    # Exit non-zero on failure so this can be used as a gate.
    if overall != "pass":
        raise SystemExit(1)


# =========================
# Swarm runtime (agents)
# =========================


def load_role_prompt(role: str) -> str:
    path = ROLES_DIR / f"{role}.txt"
    if not path.exists():
        die(f"Missing role prompt file: {path}")
    return read_text(path)


def extract_first_json_object(text: str) -> Optional[Dict[str, Any]]:
    # Very small parser: find the first {...} block and try to json-decode it.
    start = text.find("{")
    end = text.rfind("}")
    if start == -1 or end == -1 or end <= start:
        return None
    candidate = text[start : end + 1]
    try:
        obj = json.loads(candidate)
        if isinstance(obj, dict):
            return obj
        return None
    except Exception:
        return None


def openai_chat(*, model: str, system: str, user: str, timeout_seconds: int = 120) -> str:
    api_key = os.environ.get("AIORCH_OPENAI_API_KEY")
    if not api_key:
        die("AIORCH_OPENAI_API_KEY is not set")

    base_url = os.environ.get("AIORCH_OPENAI_BASE_URL", "https://api.openai.com")
    url = base_url.rstrip("/") + "/v1/chat/completions"

    payload = {
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        "temperature": 0.2,
    }

    req = urllib.request.Request(
        url,
        data=json.dumps(payload).encode("utf-8"),
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        },
        method="POST",
    )

    try:
        with urllib.request.urlopen(req, timeout=timeout_seconds) as resp:
            body = resp.read().decode("utf-8", errors="replace")
    except urllib.error.HTTPError as e:
        body = e.read().decode("utf-8", errors="replace") if e.fp else str(e)
        die(f"OpenAI HTTPError: {e.code} {body}")
    except Exception as e:
        die(f"OpenAI request failed: {e}")

    data = json.loads(body)
    return data["choices"][0]["message"]["content"]


def policy_check_commands(commands: List[str]) -> Dict[str, Any]:
    # Minimal allow/deny policy for autonomous command proposals.
    allow_prefix = ("cargo ", "python ", "rustup ", "git ")
    deny_patterns = [r"\brm\b", r"\bdel\b", r"Remove-Item", r"format\s+", r"diskpart", r"reg\s+delete"]

    blocked: List[Dict[str, str]] = []
    allowed: List[str] = []

    for cmd in commands:
        c = cmd.strip()
        if not c:
            continue
        if not c.lower().startswith(tuple(p.strip().lower() for p in allow_prefix)):
            blocked.append({"cmd": c, "reason": "prefix_not_allowlisted"})
            continue
        if any(re.search(p, c, flags=re.IGNORECASE) for p in deny_patterns):
            blocked.append({"cmd": c, "reason": "matched_deny_pattern"})
            continue
        allowed.append(c)

    return {"allowed": allowed, "blocked": blocked}


def run_role(
    *,
    role: str,
    work_order_id: str,
    work_order: Dict[str, Any],
    provider: str,
    model: str,
    artifact_dir: pathlib.Path,
    extra_context: Optional[str] = None,
) -> Dict[str, Any]:
    prompt = load_role_prompt(role)

    user_blob = {
        "work_order": work_order,
        "repo_root": str(REPO_ROOT),
    }
    user_text = "WorkOrder input (JSON):\n" + json.dumps(user_blob, indent=2)
    if extra_context:
        user_text += "\n\nAdditional context:\n" + extra_context

    if provider == "none":
        # Offline mode: produce a stub object so the pipeline still functions.
        stub = {"role": role, "work_order_id": work_order_id, "summary": "offline stub (no LLM provider configured)"}
        write_json(artifact_dir / f"{role}.json", stub)
        return stub

    if provider == "openai":
        raw = openai_chat(model=model, system=prompt, user=user_text)
    else:
        die(f"Unknown provider: {provider}")

    write_text(artifact_dir / f"{role}.response.txt", raw)
    parsed = extract_first_json_object(raw)
    if not parsed:
        parsed = {"role": role, "work_order_id": work_order_id, "summary": "failed to parse JSON from model", "raw": raw}

    write_json(artifact_dir / f"{role}.json", parsed)
    return parsed


def cmd_swarm_run(args: argparse.Namespace) -> None:
    ensure_dirs()
    work_order = load_work_order(args.work_order)

    run_id = make_run_id()
    artifact_dir = AI_DIR / "artifacts" / run_id / args.work_order / "swarm"
    artifact_dir.mkdir(parents=True, exist_ok=True)

    write_json(artifact_dir / 'manifest.json', {
        'schema_version': 'artifact_manifest.v1',
        'kind': 'swarm',
        'run_id': run_id,
        'work_order_id': args.work_order,
        'created_at': utc_now_iso(),
    })

    provider = args.provider
    model = args.model

    # Run architect + verifier in parallel; implementer consumes both.
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(2, args.parallel)) as pool:
        fut_arch = pool.submit(
            run_role,
            role="architect",
            work_order_id=args.work_order,
            work_order=work_order,
            provider=provider,
            model=model,
            artifact_dir=artifact_dir,
        )
        fut_ver = pool.submit(
            run_role,
            role="verifier",
            work_order_id=args.work_order,
            work_order=work_order,
            provider=provider,
            model=model,
            artifact_dir=artifact_dir,
        )
        architect_out = fut_arch.result()
        verifier_out = fut_ver.result()

    extra = "Architect output:\n" + json.dumps(architect_out, indent=2) + "\n\nVerifier output:\n" + json.dumps(verifier_out, indent=2)
    implementer_out = run_role(
        role="implementer",
        work_order_id=args.work_order,
        work_order=work_order,
        provider=provider,
        model=model,
        artifact_dir=artifact_dir,
        extra_context=extra,
    )

    policy = policy_check_commands(implementer_out.get("commands_to_run", []) if isinstance(implementer_out, dict) else [])
    write_json(artifact_dir / "policy_report.json", policy)

    summary = {
        "schema_version": "swarm_run.v1",
        "work_order_id": args.work_order,
        "run_id": run_id,
        "provider": provider,
        "model": model,
        "artifact_dir": str(artifact_dir.relative_to(REPO_ROOT)).replace("\\", "/"),
        "agents": {
            "architect": architect_out,
            "verifier": verifier_out,
            "implementer": implementer_out,
        },
        "policy": policy,
        "created_at": utc_now_iso(),
    }

    write_json(artifact_dir / "swarm_summary.json", summary)

    print(f"[aiorch] Swarm run complete: work_order={args.work_order} provider={provider} model={model}")
    print(f"[aiorch] Swarm artifacts: {artifact_dir}")
    print(f"[aiorch] Policy report: {artifact_dir / 'policy_report.json'}")


# =========================
# Canary runner
# =========================


def cmd_canary_run(args: argparse.Namespace) -> None:
    """Build and run the smoke harness and emit a canary health report."""
    ensure_dirs()

    run_id = make_run_id()
    artifact_dir = AI_DIR / "artifacts" / run_id / "canary"
    artifact_dir.mkdir(parents=True, exist_ok=True)

    write_json(artifact_dir / 'manifest.json', {
        'schema_version': 'artifact_manifest.v1',
        'kind': 'canary',
        'run_id': run_id,
        'created_at': utc_now_iso(),
    })

    profile = args.profile
    duration_ms = int(args.duration_ms)

    build_cmd = "cargo build -p hiwave-smoke --release" if profile == "release" else "cargo build -p hiwave-smoke"
    exe = REPO_ROOT / "target" / profile / ("hiwave-smoke.exe" if os.name == "nt" else "hiwave-smoke")

    gates: List[Dict[str, Any]] = []
    gates.append(run_gate(gate_id="build", cmd=build_cmd, timeout_seconds=3600, artifact_dir=artifact_dir))

    if gates[-1]["exit_code"] == 0:
        run_cmd = f"\"{exe}\" --duration-ms {duration_ms}"
        gates.append(
            run_gate(
                gate_id="run",
                cmd=run_cmd,
                timeout_seconds=max(30, (duration_ms // 1000) + 20),
                artifact_dir=artifact_dir,
            )
        )

    overall = "pass" if all(g["exit_code"] == 0 for g in gates) else "fail"

    report: Dict[str, Any] = {
        "schema_version": "canary_report.v1",
        "status": overall,
        "run_id": run_id,
        "profile": profile,
        "duration_ms": duration_ms,
        "started_at": utc_now_iso(),
        "gates": gates,
        "artifacts": {
            "artifact_dir": str(artifact_dir.relative_to(REPO_ROOT)).replace("\\", "/"),
            "logs": [g["stdout_path"] for g in gates] + [g["stderr_path"] for g in gates],
        },
    }

    report_json = json.dumps(report, indent=2)
    report_hash = sha256(report_json.encode("utf-8")).hexdigest()
    signed_report = {**report, "signature": {"type": "sha256", "value": report_hash}}

    report_path = AI_DIR / "reports" / f"{run_id}_canary-runner.canary.json"
    write_json(report_path, signed_report)

    print(f"[aiorch] Canary run complete: status={overall} profile={profile}")
    print(f"[aiorch] CanaryReport: {report_path}")
    print(f"[aiorch] Artifact dir: {artifact_dir}")
    print(f"[aiorch] Signature (sha256): {report_hash}")
    emit_metric({
        "event_type": "canary_run",
        "run_id": run_id,
        "status": overall,
        "profile": profile,
        "duration_ms": duration_ms,
    })

    if overall != "pass":
        raise SystemExit(1)

# =========================
# Repo integration bot
# =========================


def git(args: List[str], *, check: bool = True) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["git", *args],
        cwd=str(REPO_ROOT),
        capture_output=True,
        text=True,
        check=check,
    )


def git_ok(args: List[str]) -> bool:
    try:
        git(args, check=True)
        return True
    except Exception:
        return False


def cmd_repo_status(_: argparse.Namespace) -> None:
    try:
        out = git(["status", "--porcelain"], check=True).stdout
    except subprocess.CalledProcessError as e:
        die(f"git status failed: {e.stderr}")

    dirty = bool(out.strip())
    branch = git(["rev-parse", "--abbrev-ref", "HEAD"], check=True).stdout.strip()
    print(f"[aiorch] git branch: {branch}")
    print(f"[aiorch] working tree clean: {not dirty}")
    if dirty:
        print(out)


def cmd_repo_start(args: argparse.Namespace) -> None:
    branch = args.branch
    if not branch:
        run_id = make_run_id()
        branch = f"agent/{args.work_order}/{run_id}"

    # Ensure clean working tree.
    if git(["status", "--porcelain"], check=True).stdout.strip():
        die("Working tree is dirty; commit/stash before starting a new branch")

    # Create branch.
    try:
        git(["checkout", "-b", branch], check=True)
    except subprocess.CalledProcessError as e:
        die(f"git checkout -b failed: {e.stderr}")

    print(f"[aiorch] Created and switched to branch: {branch}")


def cmd_repo_commit(args: argparse.Namespace) -> None:
    msg = args.message
    if not msg:
        msg = f"{args.work_order}: changes"

    try:
        git(["add", "-A"], check=True)
        git(["commit", "-m", msg], check=True)
    except subprocess.CalledProcessError as e:
        die(f"git commit failed: {e.stderr}")

    print(f"[aiorch] Committed: {msg}")


def has_remote() -> bool:
    try:
        out = git(["remote"], check=True).stdout
        return bool(out.strip())
    except Exception:
        return False


def gh_available() -> bool:
    try:
        subprocess.run(["gh", "--version"], capture_output=True, text=True, check=True)
        return True
    except Exception:
        return False


def default_base_branch() -> str:
    if git_ok(['show-ref', '--verify', 'refs/heads/master']):
        return 'master'
    if git_ok(['show-ref', '--verify', 'refs/heads/main']):
        return 'main'
    # Fallback: keep existing behavior (best effort)
    return 'master'

def cmd_repo_propose(args: argparse.Namespace) -> None:
    # If gh + remote exist, create a PR. Otherwise create a local patch artifact.
    branch = git(["rev-parse", "--abbrev-ref", "HEAD"], check=True).stdout.strip()

    if has_remote() and gh_available() and not args.force_patch:
        title = args.title or f"{args.work_order}: proposed changes"
        body = args.body or "Autogenerated by aiorch. Evidence is stored under .ai/reports and .ai/artifacts."
        try:
            subprocess.run(
                ["gh", "pr", "create", "--title", title, "--body", body],
                cwd=str(REPO_ROOT),
                capture_output=True,
                text=True,
                check=True,
            )
        except subprocess.CalledProcessError as e:
            die(f"gh pr create failed: {e.stderr}\n{e.stdout}")
        print(f"[aiorch] Created PR from branch {branch}")
        return

    # Patch queue fallback
    patch_dir = AI_DIR / "artifacts" / "patch_queue"
    patch_dir.mkdir(parents=True, exist_ok=True)
    patch_path = patch_dir / f"{branch.replace('/', '_')}.patch"

    try:
        diff = git(["diff", "--binary", f"{(args.base_branch or default_base_branch())}...HEAD" ], check=True).stdout
    except subprocess.CalledProcessError as e:
        die(f"git diff failed: {e.stderr}")

    write_text(patch_path, diff)
    print(f"[aiorch] Wrote patch (no remote/gh): {patch_path}")


def find_latest_verification_report(work_order_id: str) -> Optional[pathlib.Path]:
    reports = list((AI_DIR / "reports").glob(f"*_{work_order_id}.verification.json"))
    if not reports:
        return None
    reports.sort(key=lambda p: p.stat().st_mtime, reverse=True)
    return reports[0]

def find_latest_canary_report() -> Optional[pathlib.Path]:
    reports = list((AI_DIR / 'reports').glob('*_canary-runner.canary.json'))
    if not reports:
        return None
    reports.sort(key=lambda p: p.stat().st_mtime, reverse=True)
    return reports[0]


def test_policy_pass(changed_files: List[str]) -> bool:
    """Heuristic: require at least one test-related file change."""
    patterns = [r"(^|/)tests?/", r"_test\\.rs$", r"test_.*\\.rs$"]
    return any(any(re.search(p, f) for p in patterns) for f in changed_files)

def cmd_repo_auto_merge(args: argparse.Namespace) -> None:
    # Guardrail: explicit confirmation for dangerous operation.
    if not args.yes:
        die("Refusing to auto-merge without --yes")

    # Policy: require latest VerificationReport for work_order to be pass.
    report_path = pathlib.Path(args.verification_report) if args.verification_report else find_latest_verification_report(args.work_order)
    if not report_path or not report_path.exists():
        die("No VerificationReport found; run `aiorch ci run` first")

    report = read_json(report_path)
    if report.get("work_order_id") != args.work_order or report.get("status") != "pass":
        die(f"VerificationReport is not passing for work_order={args.work_order}: {report_path}")

    # Policy: require a passing canary unless explicitly skipped.
    if not args.skip_canary:
        canary_path = pathlib.Path(args.canary_report) if args.canary_report else find_latest_canary_report()
        if not canary_path or not canary_path.exists():
            die("No CanaryReport found; run `aiorch canary run` first")
        canary = read_json(canary_path)
        if canary.get("status") != "pass":
            die(f"CanaryReport is not passing: {canary_path}")

    # Require clean working tree before merge.
    if git(["status", "--porcelain"], check=True).stdout.strip():
        die("Working tree is dirty; cannot auto-merge")

    current_branch = git(["rev-parse", "--abbrev-ref", "HEAD"], check=True).stdout.strip()
    target = args.target_branch or default_base_branch()

    # Ensure target branch exists locally.
    if not git_ok(["show-ref", "--verify", f"refs/heads/{target}"]):
        die(f"Target branch does not exist locally: {target}")

    # Guardrail: enforce per-WorkOrder file scope (if specified).
    wo = load_work_order(args.work_order)
    scope = wo.get("scope", {}) if isinstance(wo, dict) else {}
    allowed = scope.get("allowed_paths", []) or []
    forbidden = scope.get("forbidden_paths", []) or []
    changed = git(["diff", "--name-only", f"{target}...{current_branch}"], check=True).stdout.splitlines()

    if args.skip_test_policy:
        if not args.test_waiver_reason.strip():
            die("--skip-test-policy requires --test-waiver-reason")
    else:
        if not test_policy_pass(changed):
            die(
                "Test-first policy violation: no test-related files changed (use --skip-test-policy with a waiver reason to override)"
            )
    if not args.allow_lockfile and any(p in ("Cargo.lock",) for p in changed):
        die("Merge touches Cargo.lock but --allow-lockfile was not provided")

    if allowed:
        for p in changed:
            if not any(p.startswith(prefix) for prefix in allowed):
                die(f"Merge touches file outside allowed_paths: {p}")

    for p in changed:
        if any(p.startswith(prefix) for prefix in forbidden):
            die(f"Merge touches forbidden_paths file: {p}")

    # Merge current branch into target.
    try:
        git(["checkout", target], check=True)
        git(["merge", "--no-ff", current_branch, "-m", f"Merge {current_branch}"], check=True)
    except subprocess.CalledProcessError as e:
        die(f"git merge failed: {e.stderr}\n{e.stdout}")

    print(f"[aiorch] Auto-merged {current_branch} -> {target} (gated by {report_path})")

# =========================
# Auto-revert + bisect
# =========================


def require_clean_worktree() -> None:
    if git(["status", "--porcelain"], check=True).stdout.strip():
        die("Working tree is dirty; bisect/revert requires a clean worktree")


def parse_first_bad_commit(text: str) -> Optional[str]:
    m = re.search(r"(?m)^([0-9a-f]{7,40}) is the first bad commit", text)
    return m.group(1) if m else None


def cmd_bisect_canary(args: argparse.Namespace) -> None:
    """Run git bisect using the canary runner as the test command."""
    ensure_dirs()
    require_clean_worktree()

    run_id = make_run_id()
    artifact_dir = AI_DIR / "artifacts" / run_id / "bisect"
    artifact_dir.mkdir(parents=True, exist_ok=True)

    write_json(artifact_dir / 'manifest.json', {
        'schema_version': 'artifact_manifest.v1',
        'kind': 'bisect',
        'run_id': run_id,
        'created_at': utc_now_iso(),
    })

    good = args.good
    bad = args.bad
    profile = args.profile
    duration_ms = int(args.duration_ms)

    canary_cmd = [
        "python",
        str(pathlib.Path("tools/ai-orchestrator/aiorch.py")),
        "canary",
        "run",
        "--profile",
        profile,
        "--duration-ms",
        str(duration_ms),
    ]

    output_combined = ""
    culprit: Optional[str] = None

    try:
        git(["bisect", "start"], check=True)
        git(["bisect", "bad", bad], check=True)
        git(["bisect", "good", good], check=True)

        proc = subprocess.run(
            ["git", "bisect", "run", *canary_cmd],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
        )
        output_combined = (proc.stdout or "") + "\n" + (proc.stderr or "")
        write_text(artifact_dir / "bisect.stdout.txt", proc.stdout or "")
        write_text(artifact_dir / "bisect.stderr.txt", proc.stderr or "")
        culprit = parse_first_bad_commit(output_combined)
    finally:
        # Always reset bisect to restore original HEAD.
        subprocess.run(["git", "bisect", "reset"], cwd=str(REPO_ROOT), capture_output=True, text=True)

    report = {
        "schema_version": "bisect_report.v1",
        "run_id": run_id,
        "status": "pass" if culprit else "fail",
        "good": good,
        "bad": bad,
        "culprit": culprit,
        "canary": {"profile": profile, "duration_ms": duration_ms},
        "artifacts": {
            "artifact_dir": str(artifact_dir.relative_to(REPO_ROOT)).replace("\\", "/"),
            "stdout": str((artifact_dir / "bisect.stdout.txt").relative_to(REPO_ROOT)).replace("\\", "/"),
            "stderr": str((artifact_dir / "bisect.stderr.txt").relative_to(REPO_ROOT)).replace("\\", "/"),
        },
        "created_at": utc_now_iso(),
    }

    report_path = AI_DIR / "reports" / f"{run_id}_bisect.report.json"
    write_json(report_path, report)

    if culprit:
        print(f"[aiorch] Bisect complete: culprit={culprit}")
        print(f"[aiorch] BisectReport: {report_path}")
    else:
        print(f"[aiorch] Bisect failed to identify culprit")
        print(f"[aiorch] BisectReport: {report_path}")
        raise SystemExit(1)


def cmd_auto_revert(args: argparse.Namespace) -> None:
    """Bisect to find culprit, revert it on target branch, then re-run canary."""
    ensure_dirs()
    require_clean_worktree()

    target = args.target_branch or default_base_branch()
    profile = args.profile
    duration_ms = int(args.duration_ms)

    # Find culprit
    bisect_ns = argparse.Namespace(good=args.good, bad=args.bad, profile=profile, duration_ms=duration_ms)
    # cmd_bisect_canary prints; it also writes a report. We re-run its logic inline by invoking git bisect.

    # Reuse bisect command and parse the latest report it writes.
    cmd_bisect_canary(bisect_ns)

    # Find the latest bisect report for this run (best-effort)
    reports = sorted((AI_DIR / "reports").glob("*_bisect.report.json"), key=lambda p: p.stat().st_mtime, reverse=True)
    if not reports:
        die("No bisect report found")
    last = read_json(reports[0])
    culprit = last.get("culprit")
    if not culprit:
        die("Bisect did not produce a culprit")

    # Revert on target
    try:
        git(["checkout", target], check=True)
        git(["revert", "--no-edit", culprit], check=True)
    except subprocess.CalledProcessError as e:
        die(f"git revert failed: {e.stderr}\n{e.stdout}")

    # Canary validation
    canary_proc = subprocess.run(
        [
            "python",
            str(pathlib.Path("tools/ai-orchestrator/aiorch.py")),
            "canary",
            "run",
            "--profile",
            profile,
            "--duration-ms",
            str(duration_ms),
        ],
        cwd=str(REPO_ROOT),
        capture_output=True,
        text=True,
    )
    if canary_proc.returncode != 0:
        die(f"Canary still failing after revert. stdout:\n{canary_proc.stdout}\nstderr:\n{canary_proc.stderr}")

    # Emit a follow-up fix WorkOrder placeholder
    wo_id = f"fix-{culprit[:8]}"
    wo_path = work_order_path(wo_id)
    if not wo_path.exists():
        work_order = {
            "schema_version": "work_order.v1",
            "id": wo_id,
            "title": f"Investigate regression caused by {culprit[:8]}",
            "status": "pending",
            "depends_on": [],
            "goal": "Root-cause the regression found by canary/bisect and add a regression test.",
            "acceptance": {"gates": [{"id": "ci", "type": "custom", "cmd": "python tools/ai-orchestrator/aiorch.py ci run --work-order local-ci-emulator", "timeout_seconds": 7200}]},
            "steps": ["Reproduce canary failure", "Fix root cause", "Add regression test", "Verify CI + canary"],
            "rollback": {"strategy": "Revert the breaking commit or the fix if needed."},
            "created_at": utc_now_iso(),
            "updated_at": utc_now_iso(),
        }
        write_json(wo_path, work_order)

    print(f"[aiorch] Auto-revert complete: reverted={culprit} on {target}")
    print(f"[aiorch] Follow-up WorkOrder: {wo_path}")

# =========================
# Metrics
# =========================


def emit_metric(event: Dict[str, Any]) -> None:
    """Append a single JSONL metric event under .ai/reports/metrics.jsonl."""
    ensure_dirs()
    path = AI_DIR / "reports" / "metrics.jsonl"
    payload = {"ts": utc_now_iso(), **event}
    with path.open("a", encoding="utf-8") as f:
        f.write(json.dumps(payload) + "\n")


def cmd_metrics_summary(args: argparse.Namespace) -> None:
    ensure_dirs()
    path = AI_DIR / "reports" / "metrics.jsonl"
    if not path.exists():
        print("[aiorch] No metrics.jsonl found")
        return

    counts: Dict[str, Dict[str, int]] = {}
    with path.open("r", encoding="utf-8-sig") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                evt = json.loads(line)
            except Exception:
                continue
            et = str(evt.get("event_type", "unknown"))
            st = str(evt.get("status", "unknown"))
            counts.setdefault(et, {})
            counts[et][st] = counts[et].get(st, 0) + 1

    print("[aiorch] Metrics summary")
    for et in sorted(counts.keys()):
        inner = counts[et]
        parts = ", ".join(f"{k}={inner[k]}" for k in sorted(inner.keys()))
        print(f"- {et}: {parts}")

# =========================
# CLI
# =========================


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(prog="aiorch", description="Local-first AI orchestration utilities")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_init = sub.add_parser("init", help="Create repo-local .ai/ directories and a skeleton roadmap index.")
    p_init.set_defaults(func=cmd_init)

    p_val = sub.add_parser("validate-roadmap", help="Validate .ai/roadmap_index.json (lightweight).")
    p_val.set_defaults(func=cmd_validate_roadmap)

    p_ci = sub.add_parser("ci", help="Local CI emulator commands.")
    ci_sub = p_ci.add_subparsers(dest="ci_cmd", required=True)

    p_ci_run = ci_sub.add_parser("run", help="Run acceptance gates for a WorkOrder and emit a signed VerificationReport.")
    p_ci_run.add_argument("--work-order", required=True, help="Work order id (maps to .ai/work_orders/<id>.json)")
    p_ci_run.add_argument("--fail-fast", action="store_true", help="Stop at first failing gate.")
    p_ci_run.set_defaults(func=cmd_ci_run)

    p_swarm = sub.add_parser("swarm", help="Multi-agent planning commands (LLM-backed).")
    swarm_sub = p_swarm.add_subparsers(dest="swarm_cmd", required=True)

    p_swarm_run = swarm_sub.add_parser("run", help="Run architect+verifier+implementer agents and save structured handoffs.")
    p_swarm_run.add_argument("--work-order", required=True)
    p_swarm_run.add_argument("--provider", choices=["none", "openai"], default="none")
    p_swarm_run.add_argument("--model", default=os.environ.get("AIORCH_OPENAI_MODEL", "gpt-4o-mini"))
    p_swarm_run.add_argument("--parallel", type=int, default=2)
    p_swarm_run.set_defaults(func=cmd_swarm_run)

    p_canary = sub.add_parser("canary", help="Local canary runner (build + smoke harness).")
    canary_sub = p_canary.add_subparsers(dest="canary_cmd", required=True)

    p_canary_run = canary_sub.add_parser("run", help="Build and run hiwave-smoke to validate resize/multi-view flows.")
    p_canary_run.add_argument("--profile", choices=["debug", "release"], default="release")
    p_canary_run.add_argument("--duration-ms", type=int, default=4000)
    p_canary_run.set_defaults(func=cmd_canary_run)
    p_repo = sub.add_parser("repo", help="Repo integration bot (git/PR automation).")
    repo_sub = p_repo.add_subparsers(dest="repo_cmd", required=True)

    p_repo_status = repo_sub.add_parser("status", help="Show git branch + clean/dirty status.")
    p_repo_status.set_defaults(func=cmd_repo_status)

    p_repo_start = repo_sub.add_parser("start", help="Create and checkout an agent branch for a WorkOrder.")
    p_repo_start.add_argument("--work-order", required=True)
    p_repo_start.add_argument("--branch", default="")
    p_repo_start.set_defaults(func=cmd_repo_start)

    p_repo_commit = repo_sub.add_parser("commit", help="Stage all changes and commit.")
    p_repo_commit.add_argument("--work-order", required=True)
    p_repo_commit.add_argument("--message", default="")
    p_repo_commit.set_defaults(func=cmd_repo_commit)

    p_repo_propose = repo_sub.add_parser("propose", help="Create a PR via gh if available; otherwise write a patch file.")
    p_repo_propose.add_argument("--work-order", required=True)
    p_repo_propose.add_argument("--title", default="")
    p_repo_propose.add_argument("--body", default="")
    p_repo_propose.add_argument("--base-branch", default="")
    p_repo_propose.add_argument("--force-patch", action="store_true")
    p_repo_propose.set_defaults(func=cmd_repo_propose)

    p_repo_merge = repo_sub.add_parser("auto-merge", help="Auto-merge current branch into target (requires passing VerificationReport).")
    p_repo_merge.add_argument("--work-order", required=True)
    p_repo_merge.add_argument("--target-branch", default="")
    p_repo_merge.add_argument("--verification-report", default="")
    p_repo_merge.add_argument("--canary-report", default="")
    p_repo_merge.add_argument("--skip-canary", action="store_true")
    p_repo_merge.add_argument("--allow-lockfile", action="store_true")
    p_repo_merge.add_argument("--yes", action="store_true")
    p_repo_merge.set_defaults(func=cmd_repo_auto_merge)

    p_metrics = sub.add_parser("metrics", help="Metrics and throughput summaries.")
    metrics_sub = p_metrics.add_subparsers(dest="metrics_cmd", required=True)

    p_metrics_summary = metrics_sub.add_parser("summary", help="Print a summary of recorded metrics.")
    p_metrics_summary.set_defaults(func=cmd_metrics_summary)

    p_bisect = sub.add_parser("bisect", help="Run git bisect using canary as the test command.")
    bisect_sub = p_bisect.add_subparsers(dest="bisect_cmd", required=True)

    p_bisect_canary = bisect_sub.add_parser("canary", help="Bisect between --good and --bad using the canary runner.")
    p_bisect_canary.add_argument("--good", required=True)
    p_bisect_canary.add_argument("--bad", required=True)
    p_bisect_canary.add_argument("--profile", choices=["debug", "release"], default="debug")
    p_bisect_canary.add_argument("--duration-ms", type=int, default=1500)
    p_bisect_canary.set_defaults(func=cmd_bisect_canary)

    p_autorevert = sub.add_parser("auto-revert", help="Bisect to find culprit, revert it, and re-run canary.")
    p_autorevert.add_argument("--good", required=True)
    p_autorevert.add_argument("--bad", required=True)
    p_autorevert.add_argument("--target-branch", default="")
    p_autorevert.add_argument("--profile", choices=["debug", "release"], default="debug")
    p_autorevert.add_argument("--duration-ms", type=int, default=1500)
    p_autorevert.set_defaults(func=cmd_auto_revert)

    args = parser.parse_args(argv)
    args.func(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())













