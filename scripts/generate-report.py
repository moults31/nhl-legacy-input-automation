#!/usr/bin/env python3
"""Merge run_log.jsonl + daemon_events.jsonl + goal.json and produce an HTML report.

Usage: python3 scripts/generate-report.py screenshots/<RUN_ID>
Output: screenshots/<RUN_ID>/report.html
"""

import json
import os
import sys
from collections.abc import Sequence
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def parse_jsonl(path: Path) -> list[dict[str, Any]]:
    entries: list[dict[str, Any]] = []
    if not path.exists():
        return entries
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line:
                entries.append(json.loads(line))
    return entries


def parse_daemon_events(path: Path) -> list[dict[str, Any]]:
    entries: list[dict[str, Any]] = []
    if not path.exists():
        return entries
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            entry = json.loads(line)
            entry["_ts_dt"] = datetime.fromisoformat(entry["ts"].replace("Z", "+00:00"))
            entries.append(entry)
    return entries


def merge(run_dir: Path) -> tuple[list[dict[str, Any]], dict[str, Any] | None, list[str]]:
    """Merge daemon events with run_log entries. Returns (steps, goal, errors)."""
    errors: list[str] = []

    daemon_events = parse_daemon_events(run_dir / "daemon_events.jsonl")
    run_log = parse_jsonl(run_dir / "run_log.jsonl")
    goal_path = run_dir / "goal.json"
    goal = None
    if goal_path.exists():
        with open(goal_path) as f:
            goal = json.load(f)

    # Index run_log by screenshot path
    log_by_screenshot: dict[str, dict[str, Any]] = {}
    for entry in run_log:
        ss = entry.get("screenshot", "")
        log_by_screenshot[ss] = entry

    steps: list[dict[str, Any]] = []
    # Map from screenshot path to step index (handle overwrites)
    screenshot_to_idx: dict[str, int] = {}
    pending_commands: list[dict[str, Any]] = []
    prev_unique_screenshot_ts: datetime | None = None

    for event in daemon_events:
        if event["event"] == "command":
            pending_commands.append(event)
        elif event["event"] == "screenshot":
            path = event.get("path", "")
            ts = event.get("_ts_dt")

            log_entry = log_by_screenshot.get(path)

            duration_s: float | None = None
            if ts and prev_unique_screenshot_ts:
                duration_s = (ts - prev_unique_screenshot_ts).total_seconds()

            if path in screenshot_to_idx:
                # Overwrite: update the existing step in-place (the last write wins)
                idx = screenshot_to_idx[path]
                steps[idx]["timestamp"] = ts.isoformat() if ts else None
                steps[idx]["daemon_commands"].extend([c["script"] for c in pending_commands])
                steps[idx]["duration_s"] = round(duration_s, 3) if duration_s is not None else None
            else:
                step = {
                    "screenshot": path,
                    "timestamp": ts.isoformat() if ts else None,
                    "duration_s": round(duration_s, 3) if duration_s is not None else None,
                    "daemon_commands": [c["script"] for c in pending_commands],
                    "logged": log_entry is not None,
                }

                if log_entry:
                    step.update({
                        "step_num": log_entry.get("step"),
                        "assessment": log_entry.get("assessment"),
                        "decision": log_entry.get("decision"),
                        "plan": log_entry.get("plan"),
                        "vision_prompt": log_entry.get("vision_prompt"),
                        "vision_response": log_entry.get("vision_response"),
                        "logged": True,
                    })
                else:
                    step["logged"] = False

                screenshot_to_idx[path] = len(steps)
                steps.append(step)

            pending_commands = []
            prev_unique_screenshot_ts = ts

    # Cross-check: any run_log entries with no matching daemon screenshot
    logged_paths = {s["screenshot"] for s in steps if s["logged"]}
    for ss, entry in log_by_screenshot.items():
        if ss not in logged_paths:
            errors.append(f"run_log entry (step {entry.get('step')}) references screenshot '{ss}' not found in daemon_events")

    # Annotate with active task from goal.json
    if goal and "tasks" in goal:
        for i, step in enumerate(steps):
            step["_task_context"] = _find_task_context(goal, steps, i)

    return steps, goal, errors


def _find_task_context(goal: dict[str, Any], steps: list[dict[str, Any]], idx: int) -> dict[str, Any] | None:
    """Heuristically determine which goal task was active for this step."""
    tasks = goal.get("tasks", [])
    if not tasks:
        return None

    step = steps[idx]
    vision_response = step.get("vision_response")
    if not vision_response:
        for j in range(idx - 1, -1, -1):
            if steps[j].get("vision_response"):
                return _find_task_context(goal, steps, j)
        return None

    screen_title = vision_response.get("screen_title", "")
    options = vision_response.get("options", [])

    for task in tasks:
        pre_title = task.get("pre_screen_title", "")
        post_title = task.get("post_screen_title", "")
        pre_screen = task.get("pre_screen", "")
        post_screen = task.get("post_screen", "")
        pre_options = task.get("pre_options", [])

        if pre_title and pre_title.lower() in screen_title.lower():
            return {"id": task["id"], "status": task.get("status"), "phase": "pre"}
        if pre_screen and pre_screen.lower() in screen_title.lower():
            return {"id": task["id"], "status": task.get("status"), "phase": "pre"}
        if post_title and post_title.lower() in screen_title.lower():
            return {"id": task["id"], "status": task.get("status"), "phase": "post"}
        if post_screen and post_screen.lower() in screen_title.lower():
            return {"id": task["id"], "status": task.get("status"), "phase": "post"}

    return None


def compute_hotspots(steps: list[dict[str, Any]], goal: dict[str, Any] | None) -> dict[str, Any]:
    """Detect hotspots without viewing any screenshots."""
    summary: dict[str, Any] = {
        "total_screenshots": len(steps),
        "logged_steps": sum(1 for s in steps if s.get("logged")),
        "unlogged_steps": sum(1 for s in steps if not s.get("logged")),
        "mismatches": 0,
        "recoveries": 0,
        "low_confidence": 0,
        "mismatch_rate": 0.0,
        "long_pauses": [],
        "recovery_spirals": [],
        "wrong_menu_entries": [],
        "timeline": [],
    }

    logged = [s for s in steps if s.get("logged")]
    if not logged:
        return summary

    summary["mismatches"] = sum(1 for s in logged if s.get("assessment") in ("mismatch", "goal_mismatch"))
    summary["recoveries"] = sum(1 for s in logged if s.get("decision") == "recover")
    summary["low_confidence"] = sum(
        1 for s in logged
        if isinstance(s.get("vision_response"), dict) and s["vision_response"].get("confidence") == "low"
    )
    summary["mismatch_rate"] = round(summary["mismatches"] / len(logged) * 100, 1)

    # Timeline: build a compact representation of match/mismatch/unlogged
    for step in steps:
        if not step.get("logged"):
            summary["timeline"].append("unlogged")
        elif step.get("assessment") in ("goal_match",):
            summary["timeline"].append("match")
        elif step.get("assessment") in ("mismatch", "goal_mismatch"):
            summary["timeline"].append("mismatch")
        elif step.get("assessment") == "inconsistent":
            summary["timeline"].append("mismatch")
        elif step.get("assessment") == "recovery":
            summary["timeline"].append("recovery")
        elif step.get("assessment") == "halt":
            summary["timeline"].append("halt")
        else:
            summary["timeline"].append("unknown")

    # Long pauses: steps where duration > 30s (agent thinking, not game loading)
    for s in steps:
        if s.get("duration_s") and s["duration_s"] > 30:
            summary["long_pauses"].append({
                "step": s.get("step_num"),
                "screenshot": s.get("screenshot"),
                "duration": s["duration_s"],
            })

    # Recovery spirals: consecutive recover decisions or sequences of B-presses without progress
    i = 0
    while i < len(steps):
        s = steps[i]
        cmds = s.get("daemon_commands", [])
        is_recover = s.get("decision") == "recover"
        has_b = any("tap(\"b\")" in c or "tap(\"B\")" in c for c in cmds)
        is_b_back = has_b and not s.get("logged")  # unlogged B-presses suggest spiraling

        if is_recover or is_b_back:
            j = i + 1
            while j < len(steps):
                next_s = steps[j]
                next_cmds = next_s.get("daemon_commands", [])
                next_recover = next_s.get("decision") == "recover"
                next_b = any("tap(\"b\")" in c or "tap(\"B\")" in c for c in next_cmds)
                if next_recover or next_b:
                    j += 1
                else:
                    break
            span = j - i
            if span >= 2:
                summary["recovery_spirals"].append({
                    "start_idx": i,
                    "end_idx": j - 1,
                    "span": span,
                    "screenshots": [steps[k]["screenshot"] for k in range(i, j)],
                })
                i = j
            else:
                i += 1
        else:
            i += 1

    # Wrong menu entries: mismatch with screen_title in completely different submenu
    for s in logged:
        if s.get("assessment") in ("mismatch", "goal_mismatch"):
            vr = s.get("vision_response", {})
            actual = vr.get("screen_title", "")
            if actual and actual != "Press Start Screen":
                summary["wrong_menu_entries"].append({
                    "step": s.get("step_num"),
                    "expected_via_goal": s.get("_task_context"),
                    "actual_screen": actual,
                    "screenshot": s.get("screenshot"),
                })

    return summary


def generate_html(steps: list[dict[str, Any]], goal: dict[str, Any] | None, summary: dict[str, Any], run_dir: str, errors: list[str]) -> str:
    """Generate a single self-contained HTML report."""

    def esc(text: str | None) -> str:
        if text is None:
            return "-"
        return str(text).replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace('"', "&quot;")

    def format_json(obj: Any) -> str:
        return json.dumps(obj, indent=2, ensure_ascii=False)

    def badge_class(assessment: str | None) -> str:
        if assessment in ("goal_match",):
            return "badge-ok"
        elif assessment in ("goal_mismatch", "mismatch"):
            return "badge-fail"
        elif assessment == "inconsistent":
            return "badge-fail"
        elif assessment == "recovery":
            return "badge-recover"
        elif assessment == "halt":
            return "badge-halt"
        return "badge-neutral"

    def badge_label(assessment: str | None) -> str:
        if assessment in ("goal_match",):
            return "MATCH"
        elif assessment in ("goal_mismatch", "mismatch"):
            return "MISMATCH"
        elif assessment == "inconsistent":
            return "INCONSISTENT"
        elif assessment == "recovery":
            return "RECOVERY"
        elif assessment == "halt":
            return "HALT"
        return "UNLOGGED"

    # Build timeline bar
    timeline_html = ""
    for i, (step, t) in enumerate(zip(steps, summary["timeline"])):
        cls = {
            "match": "tl-ok",
            "mismatch": "tl-fail",
            "recovery": "tl-recover",
            "halt": "tl-halt",
            "unlogged": "tl-unlogged",
            "unknown": "tl-unlogged",
        }.get(t, "tl-unlogged")
        label = str(step.get("step_num", i + 1))
        screenshot = step.get("screenshot", "")
        timeline_html += f'<a href="#step-{i}" class="{cls}" title="Step {label}: {t} ({screenshot})">{label}</a>'

    # Build sections
    sections_html = ""
    for i, step in enumerate(steps):
        ss_path = step.get("screenshot", "")
        ss_rel = os.path.relpath(ss_path, run_dir) if ss_path else ""
        assessment = step.get("assessment")
        decision = step.get("decision")
        plan = step.get("plan", "")
        prompt = step.get("vision_prompt", "")
        response = step.get("vision_response", {})
        inputs_raw = step.get("daemon_commands", [])
        duration = step.get("duration_s")
        logged = step.get("logged")
        task_ctx = step.get("_task_context")
        expanded = assessment in ("mismatch", "goal_mismatch", "inconsistent", "recovery", "halt") or not logged

        # Next step's screenshot and assessment
        next_ss_rel = ""
        next_assessment = ""
        next_assessment_class = ""
        if i + 1 < len(steps):
            next_ss = steps[i + 1].get("screenshot", "")
            next_ss_rel = os.path.relpath(next_ss, run_dir) if next_ss else ""
            next_assm = steps[i + 1].get("assessment")
            if next_assm:
                next_assessment = badge_label(next_assm)
                next_assessment_class = badge_class(next_assm)

        section_class = ""
        border_color = ""
        if assessment in ("mismatch", "goal_mismatch", "inconsistent"):
            section_class = "section-mismatch"
            border_color = "#e74c3c"
        elif assessment == "recovery":
            section_class = "section-recovery"
            border_color = "#e67e22"
        elif assessment == "halt":
            section_class = "section-halt"
            border_color = "#8e44ad"
        elif not logged:
            section_class = "section-unlogged"
            border_color = "#bdc3c7"

        sections_html += f"""
        <div class="step-section {section_class}" id="step-{i}" style="{'border-left-color:' + border_color if border_color else ''}">
            <div class="step-header" onclick="toggleSection(this)">
                <div class="step-title">
                    <span class="step-num">Step {step.get('step_num', i+1)}</span>
                    <span class="badge {badge_class(assessment)}">{badge_label(assessment)}</span>
                    {f'<span class="badge badge-decision">{esc(decision)}</span>' if decision else ''}
                    <span class="step-duration">{f'{duration}s' if duration is not None else '-'}</span>
                    {f'<span class="step-task-ctx">{esc(task_ctx.get("id",""))} ({esc(task_ctx.get("phase",""))})</span>' if task_ctx else ''}
                </div>
                <span class="collapse-icon">{'▼' if expanded else '▶'}</span>
            </div>
            <div class="step-body" style="{'display:block' if expanded else 'display:none'}">
                <div class="step-main">
                    <div class="screenshot-main">
                        <div class="ss-label">SCREENSHOT</div>
                        <img src="{esc(ss_rel)}" alt="Step screenshot" loading="lazy" />
                        <div class="ss-path">{esc(ss_rel)}</div>
                    </div>
                    <div class="step-meta">
                        <div class="meta-item"><strong>Assessment:</strong> {esc(assessment)}</div>
                        <div class="meta-item"><strong>Decision:</strong> {esc(decision)}</div>
                        <div class="meta-item"><strong>Duration:</strong> {esc(f'{duration}s' if duration else '-')}</div>
                        {f'<div class="meta-item"><strong>Task:</strong> {esc(task_ctx.get("id"))} ({esc(task_ctx.get("phase"))})</div>' if task_ctx else ''}
                        <div class="meta-item"><strong>Plan:</strong> <em>{esc(plan)}</em></div>
                    </div>
                </div>

                <details class="collapsible-block">
                    <summary>Vision Prompt</summary>
                    <pre class="prompt-text">{esc(prompt)}</pre>
                </details>

                <details class="collapsible-block">
                    <summary>Vision Response</summary>
                    <pre class="json-block">{esc(format_json(response))}</pre>
                </details>

                <details class="collapsible-block">
                    <summary>Inputs Sent ({len(inputs_raw)} commands)</summary>
                    <div class="inputs-list">
                        {"".join(f'<div class="input-line"><code>{esc(cmd)}</code></div>' for cmd in inputs_raw)}
                    </div>
                </details>

                <div class="next-step-section">
                    <div class="next-header">AFTER THIS STEP →</div>
                    <div class="next-content">
                        {f'<div class="screenshot-next"><img src="{esc(next_ss_rel)}" alt="Next step" loading="lazy" /><div class="ss-path">{esc(next_ss_rel)}</div></div>' if next_ss_rel else '<div class="no-next">(end of run)</div>'}
                        {f'<div class="next-assessment"><span class="badge {next_assessment_class}">{next_assessment}</span></div>' if next_assessment else ''}
                    </div>
                </div>
            </div>
        </div>"""

    # Goal context
    goal_html = ""
    if goal:
        goal_html = f"""
        <details class="goal-section" open>
            <summary>Goal: {esc(goal.get('goal', 'N/A'))}</summary>
            <div class="tasks-grid">
                {"".join(f'''<div class="task-card {'task-completed' if t.get('status') == 'completed' else ''}">
                    <strong>{esc(t.get('id', '?'))}</strong>
                    <span class="task-status">{esc(t.get('status', '?'))}</span>
                    <p>{esc(t.get('description', ''))}</p>
                    <div class="task-detail"><em>Pre:</em> {esc(t.get('pre_screen', ''))}</div>
                    <div class="task-detail"><em>Post:</em> {esc(t.get('post_screen', ''))}</div>
                </div>''' for t in goal.get('tasks', []))}
            </div>
        </details>"""

    # Errors
    errors_html = ""
    if errors:
        errors_html = f"""
        <div class="errors-section">
            <strong>⚠ {len(errors)} merge error(s):</strong>
            <ul>{"".join(f"<li>{esc(e)}</li>" for e in errors)}</ul>
        </div>"""

    # Hotspot summary
    hotspot_html = f"""
    <div class="hotspot-summary">
        <h2>Run Hotspots</h2>
        <div class="hotspot-grid">
            <div class="hotspot-card">
                <div class="hotspot-value">{summary['total_screenshots']}</div>
                <div class="hotspot-label">Total Screenshots</div>
            </div>
            <div class="hotspot-card">
                <div class="hotspot-value">{summary['logged_steps']}</div>
                <div class="hotspot-label">Logged Steps</div>
            </div>
            <div class="hotspot-card {'hotspot-warn' if summary['mismatch_rate'] > 20 else ''}">
                <div class="hotspot-value">{summary['mismatch_rate']}%</div>
                <div class="hotspot-label">Mismatch Rate</div>
            </div>
            <div class="hotspot-card">
                <div class="hotspot-value">{summary['mismatches']}</div>
                <div class="hotspot-label">Mismatches</div>
            </div>
            <div class="hotspot-card">
                <div class="hotspot-value">{summary['recoveries']}</div>
                <div class="hotspot-label">Recoveries</div>
            </div>
            <div class="hotspot-card">
                <div class="hotspot-value">{summary.get('low_confidence', 0)}</div>
                <div class="hotspot-label">Low Confidence</div>
            </div>
            <div class="hotspot-card">
                <div class="hotspot-value">{len(summary.get('recovery_spirals', []))}</div>
                <div class="hotspot-label">Recovery Spirals</div>
            </div>
        </div>
        {_format_hotspot_details(summary)}
    </div>"""

    run_name = os.path.basename(run_dir)

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <title>NHL Run Report — {esc(run_name)}</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #1a1a2e; color: #eee; line-height: 1.5; }}
        .container {{ max-width: 1200px; margin: 0 auto; padding: 20px; }}
        h1 {{ font-size: 1.5em; margin-bottom: 10px; }}
        .subtitle {{ color: #888; margin-bottom: 20px; }}

        /* Timeline */
        .timeline-bar {{ display: flex; gap: 2px; flex-wrap: wrap; margin-bottom: 30px; padding: 10px; background: #16213e; border-radius: 8px; }}
        .timeline-bar a {{ width: 28px; height: 28px; display: flex; align-items: center; justify-content: center; border-radius: 4px; text-decoration: none; font-size: 11px; font-weight: bold; color: #fff; }}
        .tl-ok {{ background: #27ae60; }}
        .tl-fail {{ background: #e74c3c; }}
        .tl-recover {{ background: #e67e22; }}
        .tl-halt {{ background: #8e44ad; }}
        .tl-unlogged {{ background: #555; }}
        .tl-ok:hover, .tl-fail:hover, .tl-recover:hover, .tl-halt:hover, .tl-unlogged:hover {{ transform: scale(1.2); }}

        /* Hotspots */
        .hotspot-summary {{ margin-bottom: 30px; }}
        .hotspot-summary h2 {{ margin-bottom: 12px; font-size: 1.1em; color: #888; }}
        .hotspot-grid {{ display: flex; gap: 12px; flex-wrap: wrap; margin-bottom: 15px; }}
        .hotspot-card {{ background: #16213e; border-radius: 8px; padding: 14px 20px; text-align: center; min-width: 100px; }}
        .hotspot-value {{ font-size: 1.8em; font-weight: bold; }}
        .hotspot-label {{ font-size: 0.75em; color: #888; margin-top: 4px; }}
        .hotspot-warn .hotspot-value {{ color: #e74c3c; }}
        .hotspot-details {{ font-size: 0.85em; color: #aaa; }}
        .hotspot-details summary {{ cursor: pointer; margin-bottom: 8px; }}
        .hotspot-details ul {{ list-style: none; padding-left: 16px; }}
        .hotspot-details li {{ margin-bottom: 4px; }}
        .hotspot-details code {{ background: #333; padding: 1px 5px; border-radius: 3px; font-size: 0.9em; }}

        /* Goal */
        .goal-section {{ background: #16213e; border-radius: 8px; padding: 16px; margin-bottom: 20px; }}
        .goal-section summary {{ cursor: pointer; font-weight: bold; font-size: 1.1em; }}
        .tasks-grid {{ display: flex; gap: 12px; flex-wrap: wrap; margin-top: 12px; }}
        .task-card {{ background: #1a1a2e; border-radius: 6px; padding: 12px; flex: 1; min-width: 200px; }}
        .task-card.task-completed {{ border-left: 3px solid #27ae60; }}
        .task-status {{ font-size: 0.7em; background: #333; padding: 2px 8px; border-radius: 10px; margin-left: 8px; }}
        .task-detail {{ font-size: 0.8em; color: #888; margin-top: 4px; }}

        /* Errors */
        .errors-section {{ background: #4a1a1a; border: 1px solid #e74c3c; border-radius: 8px; padding: 12px; margin-bottom: 20px; }}
        .errors-section ul {{ margin: 8px 0 0 20px; }}

        /* Step sections */
        .step-section {{ background: #16213e; border-radius: 8px; margin-bottom: 8px; border-left: 4px solid transparent; overflow: hidden; }}
        .step-section.section-mismatch {{ border-left-color: #e74c3c; }}
        .step-section.section-recovery {{ border-left-color: #e67e22; }}
        .step-section.section-halt {{ border-left-color: #8e44ad; }}
        .step-section.section-unlogged {{ opacity: 0.6; }}
        .step-header {{ display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; cursor: pointer; user-select: none; }}
        .step-header:hover {{ background: rgba(255,255,255,0.03); }}
        .step-title {{ display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }}
        .step-num {{ font-weight: bold; font-size: 1.05em; }}
        .badge {{ font-size: 0.7em; padding: 3px 8px; border-radius: 10px; font-weight: bold; text-transform: uppercase; }}
        .badge-ok {{ background: #27ae60; color: #fff; }}
        .badge-fail {{ background: #e74c3c; color: #fff; }}
        .badge-recover {{ background: #e67e22; color: #fff; }}
        .badge-halt {{ background: #8e44ad; color: #fff; }}
        .badge-neutral {{ background: #555; color: #ddd; }}
        .badge-decision {{ background: #2c3e50; color: #aaa; }}
        .step-duration {{ font-size: 0.8em; color: #888; }}
        .step-task-ctx {{ font-size: 0.75em; color: #666; font-style: italic; }}
        .collapse-icon {{ color: #888; font-size: 0.8em; }}
        .step-body {{ padding: 0 16px 16px; }}
        .step-main {{ display: flex; gap: 20px; margin-bottom: 16px; }}
        .screenshot-main {{ flex: 0 0 45%; }}
        .screenshot-main img {{ width: 100%; border-radius: 6px; border: 1px solid #333; }}
        .screenshot-next img {{ width: 100%; max-width: 400px; border-radius: 6px; border: 1px solid #333; }}
        .ss-label {{ font-size: 0.65em; color: #666; letter-spacing: 1px; margin-bottom: 4px; }}
        .ss-path {{ font-size: 0.7em; color: #555; margin-top: 4px; word-break: break-all; }}
        .step-meta {{ flex: 1; }}
        .meta-item {{ margin-bottom: 6px; font-size: 0.9em; }}
        .collapsible-block {{ background: #1a1a2e; border-radius: 6px; padding: 10px 14px; margin-bottom: 10px; }}
        .collapsible-block summary {{ cursor: pointer; font-weight: 600; font-size: 0.85em; color: #aaa; }}
        .collapsible-block pre {{ margin-top: 8px; font-size: 0.78em; white-space: pre-wrap; word-break: break-word; color: #ccc; max-height: 400px; overflow-y: auto; }}
        .prompt-text {{ color: #aaa; }}
        .json-block {{ color: #8eb; }}
        .inputs-list {{ margin-top: 8px; max-height: 300px; overflow-y: auto; }}
        .input-line {{ padding: 4px 0; font-size: 0.8em; border-bottom: 1px solid #222; }}
        .input-line code {{ color: #e8b; }}
        .next-step-section {{ background: #1a1a2e; border-radius: 6px; padding: 12px 14px; margin-top: 12px; }}
        .next-header {{ font-size: 0.7em; color: #666; letter-spacing: 1px; margin-bottom: 8px; }}
        .next-content {{ display: flex; gap: 16px; align-items: flex-start; }}
        .no-next {{ color: #555; font-style: italic; }}

        @media (max-width: 768px) {{
            .step-main {{ flex-direction: column; }}
            .screenshot-main {{ flex: 1; }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>NHL Run Report — {esc(run_name)}</h1>
        <div class="subtitle">Generated {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</div>

        {errors_html}
        {hotspot_html}
        {goal_html}

        <h2 style="margin-bottom:10px">Run Timeline</h2>
        <div class="timeline-bar" id="timeline">{timeline_html}</div>

        <h2 style="margin-bottom:10px">Step Details ({len(steps)} total)</h2>

        <div class="step-sections">
            {sections_html}
        </div>
    </div>

    <script>
        function toggleSection(header) {{
            const body = header.nextElementSibling;
            const icon = header.querySelector('.collapse-icon');
            if (body.style.display === 'none') {{
                body.style.display = 'block';
                icon.textContent = '▼';
            }} else {{
                body.style.display = 'none';
                icon.textContent = '▶';
            }}
        }}

        document.addEventListener('keydown', function(e) {{
            if (e.key === 'ArrowDown' || e.key === 'ArrowRight') {{
                e.preventDefault();
                navigateBy(1);
            }} else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {{
                e.preventDefault();
                navigateBy(-1);
            }}
        }});

        function navigateBy(delta) {{
            const sections = document.querySelectorAll('.step-section');
            let current = -1;
            for (let i = 0; i < sections.length; i++) {{
                const header = sections[i].querySelector('.step-header');
                if (header && document.activeElement === header) {{
                    current = i;
                    break;
                }}
            }}
            if (current === -1) {{
                current = delta > 0 ? -1 : sections.length;
            }}
            const next = Math.max(0, Math.min(sections.length - 1, current + delta));
            const header = sections[next].querySelector('.step-header');
            if (header) header.focus();
            sections[next].scrollIntoView({{ behavior: 'smooth', block: 'center' }});
        }}

        // Make step headers focusable for keyboard nav
        document.querySelectorAll('.step-header').forEach(function(h) {{
            h.setAttribute('tabindex', '0');
        }});
    </script>
</body>
</html>"""


def _format_hotspot_details(summary: dict[str, Any]) -> str:
    parts: list[str] = []

    if summary.get("long_pauses"):
        items = "".join(
            f"<li>Step {p['step']}: <code>{p['screenshot']}</code> — {p['duration']:.1f}s</li>"
            for p in summary["long_pauses"]
        )
        parts.append(f"<li><strong>Long pauses (&gt;30s):</strong><ul>{items}</ul></li>")

    if summary.get("recovery_spirals"):
        items = "".join(
            f"<li>{s['span']} consecutive recover/B-press steps (steps {s['start_idx']+1}–{s['end_idx']+1})</li>"
            for s in summary["recovery_spirals"]
        )
        parts.append(f"<li><strong>Recovery spirals:</strong><ul>{items}</ul></li>")

    if summary.get("wrong_menu_entries"):
        items = "".join(
            f"<li>Step {w['step']}: ended up in <code>{w['actual_screen']}</code></li>"
            for w in summary["wrong_menu_entries"]
        )
        parts.append(f"<li><strong>Wrong menu entries:</strong><ul>{items}</ul></li>")

    if summary.get("unlogged_steps", 0) > 5:
        parts.append(f"<li><strong>High unlogged rate:</strong> {summary['unlogged_steps']}/{summary['total_screenshots']} screenshots have no run_log entry</li>")

    if parts:
        return f'<details class="hotspot-details"><summary>Details</summary><ul>{"".join(parts)}</ul></details>'
    return ""


def main() -> None:
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <run_dir>", file=sys.stderr)
        print("Example: python3 scripts/generate-report.py screenshots/20260710_194059_run", file=sys.stderr)
        sys.exit(1)

    run_dir = sys.argv[1]
    run_path = Path(run_dir)
    if not run_path.is_dir():
        print(f"Error: directory not found: {run_dir}", file=sys.stderr)
        sys.exit(1)

    steps, goal, errors = merge(run_path)
    if not steps:
        print("Warning: no steps found in daemon events", file=sys.stderr)

    summary = compute_hotspots(steps, goal)
    html = generate_html(steps, goal, summary, run_dir, errors)

    report_path = run_path / "report.html"
    with open(report_path, "w") as f:
        f.write(html)

    print(f"Report written: {report_path}")
    print()

    # Print hotspot summary to stdout (for agents to consume)
    print("=== HOTSPOT SUMMARY ===")
    print(f"Total screenshots: {summary['total_screenshots']}")
    print(f"Logged steps:      {summary['logged_steps']}")
    print(f"Unlogged steps:    {summary['unlogged_steps']}")
    print(f"Mismatches:        {summary['mismatches']} ({summary['mismatch_rate']}%)")
    print(f"Recoveries:        {summary['recoveries']}")
    print(f"Low confidence:    {summary.get('low_confidence', 0)}")
    print(f"Recovery spirals:  {len(summary.get('recovery_spirals', []))}")
    print(f"Long pauses:       {len(summary.get('long_pauses', []))}")
    print(f"Wrong menu entries:{len(summary.get('wrong_menu_entries', []))}")

    if summary.get("long_pauses"):
        print("\nLong pauses (>10s):")
        for p in summary["long_pauses"]:
            print(f"  Step {p['step']}: {p['screenshot']} — {p['duration']:.1f}s")

    if summary.get("recovery_spirals"):
        print("\nRecovery spirals:")
        for s in summary["recovery_spirals"]:
            ss_list = ", ".join(s["screenshots"])
            print(f"  {s['span']} consecutive recover/B-press steps: {ss_list}")

    if errors:
        print(f"\nErrors ({len(errors)}):")
        for e in errors:
            print(f"  - {e}")

    summary_path = run_path / "hotspot_summary.json"
    with open(summary_path, "w") as f:
        json.dump(summary, f, indent=2, default=str)
    print(f"\nHotspot summary written: {summary_path}")


if __name__ == "__main__":
    main()
