# Build your own pair-programming agent

*A step-by-step guide to a Rust-powered coding workspace where you can edit by hand, collaborate with an agent, or delegate a bounded task—and use a real browser or test device along the way.*

## First release · Make it useful before the PKM

Use your personal website as the first real testbed. Ship a **small, preview-only Pair mode**, then move on to the PKM. The full workspace below is the roadmap, not a prerequisite.

1. **Connect stock Zed to a Rust agent over ACP.** Complete Milestone 0 and the preview/staleness fixture in Milestone 1. No fork and no write tool: show a diff for manual application, and confirm that an unsaved edit cannot be overwritten by the agent.
2. **Investigate one real website issue.** Reproduce it yourself in the browser and paste the relevant error or observation into the session. Use scoped file reads and exact search with file/line references, one real model provider, and one `fix-regression` skill. Take only those pieces from Milestones 2–3; skip indexing, symbol graphs, and browser automation for now.
3. **Verify your version.** Apply or rewrite the suggested change yourself, save it, and explicitly approve a named project check. Report the command, outcome, and saved state checked; do not imply unsaved text was tested. If the check cannot run, report that as unverified. Without process isolation, only run fixed checks in your own trusted repo and do not present the allowlist as a sandbox.
4. **Decide whether to keep using it.** Compare a few representative tasks with a prebuilt agent in Zed, using equivalent starting code and checks. Record time to verified fix, manual context and cleanup, and failed or unverified attempts. Keep it if it saves effort or its handoff is worth maintaining. Either way, start the PKM after this trial; use a prebuilt agent if yours isn't ready.

**Defer:** Playwright control, richer retrieval, Auto mode, Android/iOS, MCP, and editor forking. Pull them back into scope when real project work shows a need. The PKM and any later work on Marmat Sewa or Amber Alert can use the agent, but none should depend on it.

## What you’re building

**One workspace, three collaboration modes.** The editor, Git checkout, browser, and test devices are useful without AI. In Pair mode, you hand the agent a specific part of the work: an error to investigate, a patch to propose, or a test to run. You can take that work back and ask it to verify your version. Auto mode delegates a larger task against an explicit baseline in a separate checkout. These are permission presets over the same engine, not three separate agents.

Build the harness, retrieval engine, and skills yourself so you can change how the system works. Reuse Git, the editor, browser automation, device tools, and the MCP SDK. Start with **stock Zed and a Rust CLI/core with a thin ACP adapter**, not a new IDE. The agent does not need to be the only way to use the workspace.

| Mode | What the developer does | Agent boundary |
| --- | --- | --- |
| **Manual** | Edit, use the browser, launch a test app, run commands. | No model requests, background agent actions, or automatic sharing. Local indexing is separate and can be disabled. |
| **Pair** | Share selected evidence, review a patch, edit it, or take over a surface. | Read scoped context, propose changes, and run approved checks. Drive only an explicitly delegated browser tab or device. |
| **Auto** | Select a task, base commit, and budget; review the result. | Write inside a designated worktree, use only permitted capabilities, stop on limits, and return a diff plus verification. No auto-merge or push in the first release. |

### The demo to build toward

Open a small web project. Reproduce a bug by hand in a visible browser, then share a console error and the relevant element with the agent. It retrieves the relevant code and test, explains what it found, and proposes a diff. You edit the file yourself, then click **Verify current changes**. The agent verifies what you actually saved and shows which revision was tested. Repeat with an unsaved edit and an intervening disk edit: neither should be silently overwritten by an old proposal. Finally, delegate the same task to Auto against a clean base and inspect its result without it touching your working checkout.

## Stack and component boundaries

| Layer | Starting choice | What you own |
| --- | --- | --- |
| Native runtime | Rust stable, Cargo, Tokio, Serde, `tracing` | Session state, permissions, scheduling, cancellation, tool dispatch, and audit events. |
| Interface | Stock Zed Agent Panel; Rust CLI for direct use | Zed owns editing. Your agent exposes session updates, evidence, and proposals. Do not assume editor buffer events or custom panels are available. |
| Core ↔ editor | Rust binary with an Agent Client Protocol (ACP) adapter over stdio | Keep ACP session and capability handling at the boundary; the core stays editor-independent. MCP remains optional for tool interoperability. |
| Repository and retrieval | Git CLI, SQLite FTS5 via `rusqlite`; add one Tree-sitter grammar later | File inventory, incremental indexing, symbol extraction, ranker, context budgets, and provenance. |
| Model | Rust provider interface, fixture provider first, one real provider second | Typed actions, streaming, context assembly, cost budgets, and provider-independent tests. |
| Browser | Small TypeScript/Node Playwright worker, dedicated visible Chromium session | Shareable evidence, page ownership, and bounded actions. Browser automation stays outside the Rust core. |
| Mobile | `adb` and Android Emulator; `xcrun simctl` and Xcode on macOS | Discovery, launch, evidence capture, and eventually platform-specific UI test actions. |
| Skills and MCP | Versioned `SKILL.md` files; Rust workflow rules; `rmcp` for optional MCP server/client | Skill routing and verification rules; reusable tool implementations behind an optional standard protocol. |

**Custom does not mean dependency-free.** Write the policy engine, ranking logic, handoff rules, and tooling behavior you want to control. Don’t write a browser driver, editor, Git implementation, or MCP transport just to say you own them. Rust helps with local indexing, orchestration, and responsiveness; it doesn’t speed up inference on a remote model.

---

## 0 · Scaffold a workspace you can use without AI

Create a Cargo workspace and start with a Rust CLI. Add a thin ACP entry point for stock Zed; no extension or fork is required for this checkpoint. Start with a small, synthetic web app: one page, one intentionally wrong behavior, and one regression test. For example, a quantity control allows `-1` and logs a recognizable error. Put a plausible but unrelated implementation in another file so search has something to get wrong.

```text
pair-workspace/
  Cargo.toml
  crates/
    agent-core/          # run state, policies, typed requests
    agent-cli/           # human CLI and ACP service entry point
    repo-index/          # retrieval and ranking
    tool-mcp/            # optional adapter, added later
  adapters/
    browser-worker/     # Playwright sidecar, added later
  skills/
    fix-regression/SKILL.md
  fixtures/
    web-regression/
  evals/
```

```sh
cargo new --lib crates/agent-core
cargo new --bin crates/agent-cli
cargo new --lib crates/repo-index
```

Create a root `Cargo.toml` with those members and use a single lockfile. Start with a CLI `status` command for one local trusted repository. Report the repository root, Git HEAD, and dirty state. Open the same fixture in stock Zed and register the local binary as a custom external agent using Zed’s docs. Implement ACP initialization, session creation, and a fixture prompt that reports workspace status in the Agent Panel. Reserve stdout for protocol messages and stderr for logs. Do not invoke a real model, read unsaved buffers, or write files in this milestone.

**Read:** [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html) · [Zed external agents](https://zed.dev/docs/ai/external-agents) · [ACP protocol](https://agentclientprotocol.com/protocol/v1/overview)

**Move on when:** the CLI and Zed show the status of the same fixture without a model or API key. Run Rust CI on Windows, macOS, and Linux; smoke-test launching the local agent in Zed on each supported host. Defer remote workspaces.

## 1 · Make a reviewed patch safe before connecting a model

Write a fake provider that produces a known diff. In Pair, store the diff as a proposal and show it as a fenced unified diff in the Agent Panel (or print it from the CLI). For the first implementation, **the human applies it using normal editor operations**; the agent has no write permission to the manual checkout. This is still a complete pair loop, and it avoids depending on a check-then-write sequence that can race the editor or an external process. Add one-click application only if you can enforce safe, version-aware semantics for the editor and file types you support; otherwise keep manual application.

```text
PatchProposal {
  proposal_id, run_id, workspace_id, base_head,
  touched_files: [{ repo_relative_path, base_content_hash, editor_version? }],
  unified_diff, evidence_ids
}
```

Build initial proposals from saved files. When the ACP client advertises `fs.readTextFile`, use it with the user’s consent to compare editor-visible text against the saved baseline; if it differs, mark the proposal stale and ask for a fresh review. Treat a missing capability or failed read as **buffer state unknown**, not as proof the file is clean. Keep Pair preview-only even if a read matches: standard `fs.writeTextFile` does not carry an expected revision, so read–compare–write is not an atomic buffer update. For verification, prompt the user to save, run against saved inputs only, and report the tested baseline. A pre/post hash can detect some changes, but it does not prove isolation. Later, verify in an isolated snapshot when you need that guarantee.

Put a narrow dispatcher between model actions and execution. Accept `run_check { check_id }`, not `run_shell { command }`. Resolve check IDs through a reviewed configuration to executable + argument vector + working directory, with a sanitized environment and bounded stdout/stderr. Launch without shell interpolation; treat platform wrappers like `.cmd` as a separate, tested case rather than assuming identical quoting. Running a *permitted* build or test still executes repository code and dependencies. **An allowlist is not a sandbox.** Until real isolation exists, run only in explicitly trusted repositories, with no sensitive inherited credentials and no claim of enforced filesystem or network confinement.

**Read:** [ACP filesystem capabilities](https://agentclientprotocol.com/protocol/v1/file-system) · [Zed external agents](https://zed.dev/docs/ai/external-agents) · [Tokio process management](https://docs.rs/tokio/latest/tokio/process/) · [Git apply](https://git-scm.com/docs/git-apply)

**Move on when:** the fake patch is visible, a changed saved file makes it stale, and an unsaved edit is either detected by an explicit read or remains protected by preview-only mode. Verification never reports an unsaved buffer as tested. A canceled run does not accept a late result as current. Test process-tree cleanup with a fixture that spawns grandchildren; killing the immediate child is not sufficient. Use suitable process-group handling on Unix and investigate [Windows Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects) for Windows; keep Auto disabled on a platform until its stop behavior is verified.

## 2 · Build your own repository retrieval

Start with a read-only `search_repo` and a `read_excerpt` tool. Inventory tracked files, plus explicitly enabled, non-ignored untracked files. Start with explicit reindexing, then add file watching with reconciliation for missed events. Skip binaries, oversized files, dependency trees, symlinks you can’t safely resolve, and configured sensitive paths. `.gitignore` is not a privacy policy; maintain a separate exclusion list and do not assume all secrets have predictable filenames.

```sql
CREATE VIRTUAL TABLE repo_search USING fts5(
    path UNINDEXED, symbol, body
);
```

Use a separate metadata table for content hashes, byte/line ranges, parser version, and staleness; index transactionally. Bind SQL parameters and handle FTS query syntax separately. Start with exact path, symbol-name matches, and exact error-string candidates, then use FTS5 to fill gaps. Index functions or small line windows, not entire monorepo files. Add Tree-sitter only for the language needed by the fixture or first real repository, then improve extraction based on measured misses. Syntax trees help locate declarations; they do not give you a resolved call graph by themselves.

Return `path`, line range, base hash, excerpt, and a short reason such as `exact error match` or `adjacent test`. Keep the context budget small and deterministic. Recheck a result’s hash on read; if stale, refetch or discard it. Treat repository text and every other retrieved payload as **untrusted data**, not as authorization to run commands or disclose files. Project rules that control permissions belong in separately reviewed configuration.

**Read:** [Git ls-files](https://git-scm.com/docs/git-ls-files) · [SQLite FTS5](https://sqlite.org/fts5.html) · [Tree-sitter documentation](https://tree-sitter.github.io/tree-sitter/)

**Move on when:** a fixture error retrieves the relevant symbol and regression test ahead of the distractor, with usable citations. Rename a file, change its contents, and remove it; verify old excerpts do not survive reconciliation. Measure cold indexing, incremental refresh, and query latency separately. Add embeddings only if benchmark misses justify them.

## 3 · Give the agent a skill and a bounded loop

Wire a single model provider only after fake tool calls pass. Describe operations as typed requests: `search_repo`, `read_excerpt`, `propose_patch`, `run_check`, `finish`. Reject unknown operations, invalid arguments, stale grants, and oversized responses. Treat the selected skill as workflow guidance; implement required checks in Rust. For each run, record the model, skill version, base, approved capabilities, tool attempts, and observed outcomes. Keep raw prompts and source excerpts out of default logs.

Start with explicit skill selection through a **Fix regression with me** command. Automatic selection can come after you can measure false activations. For example:

```markdown
---
name: fix-regression
description: Investigate a specific reported regression and propose a minimal fix.
---

# Fix a regression

1. Ask for or inspect shared reproduction evidence; do not treat a guess as a reproduction.
2. Retrieve the failing behavior's implementation and its closest test.
3. Propose the smallest patch that addresses the cause. Explain the test that would verify it.
4. In Pair mode, wait for the developer to review and apply changes.
5. Verify the current saved revision with approved checks. Report blocked checks separately.
6. Do not weaken existing assertions to make a test pass. Surface such changes for separate review.
```

Keep permissions outside the skill text. A `finish` request is not evidence that verification passed: derive the UI status from the runner’s result for the recorded input revision. Use explicit outcomes such as **passed**, **failed**, **not run**, and **stale**. Apply time, tool-call, output, and configured token budgets, including on failure loops.

**Read:** [Agent Skills format](https://agentskills.io/specification) · [Serde derive](https://serde.rs/derive.html) · [Rust tracing](https://docs.rs/tracing/latest/tracing/). Once you pick a provider, read its tool-calling and streaming docs and test its actual accepted schemas rather than assuming common behavior.

**Move on when:** a developer can share a test error, get a contextual patch, make a different change by hand, and verify their own result. The same flow must fail cleanly when the provider returns invalid JSON, the test is unavailable, or the developer stops the run.

## 4 · Add a visible browser and a real handoff

Start a headed Chromium browser from a small Playwright worker. Use a fresh browser context and test data, not the developer’s everyday profile. The user drives it by default. Add a **Share this tab’s evidence** action that captures only a selected console error, origin, and optionally a sanitized screenshot or selected DOM information. Filter secrets before sending data to a remote model; avoid collecting cookies, auth headers, full request bodies, or all DOM text by default.

Then implement **Give agent control** / **Take control** for one tab. Track a grant with `surface_id`, `session_id`, `generation`, allowed actions, and expiry. Check it at dispatch and again where possible before an effect. On takeback, invalidate the generation, cancel queued actions, and wait for any in-flight action to stop or settle before acknowledging handoff. If an action has already submitted a form, revocation cannot undo it. Don’t imply exclusive human control if you cannot prevent the developer from interacting directly with the browser window; show the current grant and warn on overlap. Start with disposable localhost fixtures, an explicit origin allowlist, and navigation/redirect checks. Don’t let a browser button or console message grant itself new privileges.

Begin with `get_shared_console`, `inspect_element`, and `capture_view`; add an approved `click` or `navigate` only after the read-only handoff works. Don’t expose arbitrary page evaluation to the model. Set equivalent browser boundaries when the task moves to Auto; don’t point it at the same mutable app state without an explicit reason.

**Read:** [Playwright browser contexts](https://playwright.dev/docs/browser-contexts) · [BrowserType and connection options](https://playwright.dev/docs/api/class-browsertype) · [Playwright locators](https://playwright.dev/docs/locators)

**Move on when:** you can reproduce the fixture bug by hand, hand the agent the selected evidence, take control back, and verify your edited patch. Revocation rejects a queued action from the old grant, and a malicious string in page content cannot cause arbitrary execution.

## 5 · Add bounded Auto mode using a separate worktree

Require the user to select a base commit, a task, and allowed checks. Launch a dedicated worktree at a managed path, with no automatic copy of the developer’s dirty tree or unsaved buffers. If the intended base includes manual changes, ask for a commit in v0. A later explicit snapshot/import flow can show exactly what it copies.

```sh
# Example: fully substitute an agent-owned directory and a chosen commit.
git worktree add --detach /path/to/agent-run <base-commit>
git -C /path/to/agent-run status --short
```

Have Auto evaluate and apply its own typed patches in that worktree. Reject path traversal, symlink escapes, writes to Git metadata, and writes outside the run's configured scope. Use argument arrays and verify the worktree identity before executing a check or applying a patch. Set separate app ports and browser profiles if needed. On stop, stop dispatching and clean up processes before returning a partial result. Preserve the worktree until the user reviews or discards it; don’t run destructive cleanup on a path the agent does not own.

Export a diff, a base ID, and the verification record. Surface newly created files explicitly: a plain `git diff HEAD` does not include untracked files. Never implicitly merge, commit, or push to the active checkout. A worktree isolates *where you direct edits*, not what arbitrary code can access: it shares repository metadata and does not restrict subprocesses. For untrusted projects or enforced no-network execution, add and test a real isolation backend before enabling those workflows. Make the isolation level visible rather than implying uniform guarantees across OSes.

**Read:** [Git worktree](https://git-scm.com/docs/git-worktree) · [Git diff](https://git-scm.com/docs/git-diff) · [Git status and machine-readable output](https://git-scm.com/docs/git-status)

**Move on when:** a delegated fix returns a reviewable patch without changing the manual checkout, a timeout returns a partial result, and cancellation leaves no test server running. Verify that untracked files are shown, not silently omitted, and that a dirty manual checkout stays intact.

## 6 · Add Android and iOS as capability adapters

Define a small internal device interface: `list_devices`, `inspect_state`, `launch_app`, `capture_evidence`, and, later, `run_ui_check`. Report support by capability rather than pretending every host can perform every action. Start with a single user-selected emulator and read-only evidence. Use synthetic accounts and keep logs and screenshots out of model context unless shared.

| Surface | First implementation | Later |
| --- | --- | --- |
| **Android** | Discover with `adb devices -l`, target a chosen emulator with `adb -s <serial>`, and capture a screenshot with `adb -s <serial> exec-out screencap -p`. Keep the emulator visible so the developer can reproduce manually. | Install and launch the fixture app, collect scoped logs, then add an instrumentation or UI Automator check. Refuse physical devices by default. |
| **iOS** | On macOS with Xcode and a simulator runtime, discover with `xcrun simctl list devices available -j`; target a specific UDID. Capture with `xcrun simctl io <udid> screenshot <path>` after the simulator is booted. | Launch and verify a fixture app with `simctl` and Xcode UI testing. A secured remote Mac runner is a separate project, not a requirement for the alpha. |

Use the same handoff model as the browser: a grant applies to one device, and loss of connection invalidates the grant. Do not silently target another device if the expected one disappears. For Auto, prefer a separately provisioned test device; leave device automation disabled when you cannot isolate its test state. Android tooling runs on supported Windows, Linux, and macOS hosts, subject to emulator and virtualization requirements. On Windows and Linux, present local iOS Simulator as unavailable, not broken.

**Read:** [Android Emulator command line](https://developer.android.com/studio/run/emulator-commandline) · [Android Debug Bridge](https://developer.android.com/tools/adb) · [Android UI Automator](https://developer.android.com/training/testing/other-components/ui-automator) · [Apple Xcode command-line tool reference](https://developer.apple.com/documentation/xcode/xcode-command-line-tool-reference) · [Apple XCTest](https://developer.apple.com/documentation/xctest). Check installed `xcrun simctl help` for the commands your Xcode version actually supports.

**Move on when:** a person can reproduce a bug manually, share one screenshot or relevant log, and verify a fix on the same explicitly selected emulator. Test disconnected, unauthorized, and unavailable-device cases. Run Android checks only where emulator support is configured and iOS smoke checks on a configured Mac.

## 7 · Expose one custom tool over MCP

Keep native tools as library functions. As your first local MCP server, expose `find_regression_context`: query your repository index and return a short ranked set of source and test excerpts with paths and line ranges. Back it with exactly the same implementation your agent calls directly. Start with a pinned local repository root and a read-only server over stdio. Do not expose a generic read-any-path tool or arbitrary commands. Keep protocol output on stdout and logs on stderr. Use the SDK to handle protocol lifecycle and schema negotiation rather than inventing your own MCP transport.

Separately, if your own agent needs to *consume* external MCP tools, add a client after the server works. Treat server startup configuration like installing executable code. Pin trusted commands, limit what credentials and environment they receive, review their tool inventory, and reapprove material permission changes. MCP tool descriptions and responses cannot authorize additional actions. A custom server can replace a connector whose underlying data you already have authorized access to; it cannot remove an upstream service's API restrictions or charges.

**Read:** [MCP specification](https://modelcontextprotocol.io/specification/) · [Official Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)

**Move on when:** your core and a second MCP-capable host retrieve the same facts from the same fixture, and the server refuses a request outside its root. Leave MCP optional: neither Manual mode nor the internal Pair loop should depend on it.

## 8 · Benchmark, package, and cut the alpha

Build a replay corpus before tuning: real or synthetic regression tasks with a known baseline, a test oracle, expected files, and a few no-change tasks. Include a stale buffer, misleading search result, prompt injection embedded in a fixture, unavailable emulator, and an interrupted task. Use a fake provider for deterministic security and state-machine tests; use the real provider for quality and latency. Keep a held-out set separate from tasks used to tune retrieval.

| Measure | What to record |
| --- | --- |
| Collaboration | Whether handoff succeeds, stale edits are flagged, and the developer can verify their version. |
| Retrieval | Whether the intended file or symbol appears in the first few hits; cold and warm query latency; freshness after edits. |
| Correctness | Verified fixes against an independent oracle, unintended edits, and checks incorrectly reported as passed. |
| End-to-end cost | Time to verified patch, model turns and tokens, local tool time, and test time. Treat incomplete attempts as incomplete rather than removing them from comparisons. |
| Control | Whether forbidden actions fail, cancellation works, and every tool effect has a traceable grant. |

Compare your agent with a prebuilt one on the same base commits and fixtures, using the same model and comparable permissions and budgets where possible. Record differences you cannot control. Run multiple trials where feasible. Run an ablation of simple exact search versus richer retrieval, so you know whether the custom index improves results or just adds machinery. For Pair, evaluate the turn-taking separately from autonomous fix rates; the ability to take over is part of the product.

Package the native binaries and document the Zed custom-agent setup for the supported OS/architecture combinations. Document Git, browser, and device prerequisites; don’t make optional SDKs mandatory for Manual mode. Show a capability panel (`Git available`, `browser unavailable`, `iOS requires macOS/Xcode`, etc.). Gate the release on an editor/CLI smoke test, a fake-agent Pair run, and clean cancellation on each supported host. Enable Auto and device integration only where their specific checks pass.

**Read:** [Zed external-agent setup](https://zed.dev/docs/ai/external-agents) · [Cargo build](https://doc.rust-lang.org/cargo/commands/cargo-build.html)

**Alpha boundary:** Keep full IDE replacement, mobile UI mirroring, access to everyday browser profiles, remote Mac orchestration, and arbitrary third-party skills out of scope. The core product is a handoff you can trust, not the number of tools the agent can list.

## Editor decision after the alpha

Keep stock Zed unless a specific requirement is blocked. The first candidate is a native apply operation that checks the editor-buffer revision as part of the edit itself. Prove the limitation with a fixture before proposing an upstream change or maintaining a fork. A tiny from-scratch editor is a separate learning exercise, not a prerequisite for the agent or PKM.

## Start here

Follow **First release · Make it useful before the PKM** above. Your first coding checkpoint is stock Zed → ACP → fake preview, with the agent unable to write. Your first real-use checkpoint is a verified website fix after you, not the agent, edit and save the code.
