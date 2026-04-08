# 🦞 Claw Code — Rust Implementation

A high-performance Rust rewrite of the Claw Code CLI agent harness. Built for speed, safety, and native tool execution.

## Quick Start

```bash
# Build
cd rust/
cargo build --release

# Run interactive REPL
./target/release/claw

# One-shot prompt
./target/release/claw prompt "explain this codebase"

# With specific model
./target/release/claw --model sonnet prompt "fix the bug in main.rs"
```

## Configuration

Set your API credentials:

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
# Or use a proxy
export ANTHROPIC_BASE_URL="https://your-proxy.com"
```

Or authenticate via OAuth:

```bash
claw login
```

## Providers & Auth Support Matrix

Before anything else: **know which branch you're building.** Provider
support differs between `dev/rust` and `main`, and neither branch
currently supports AWS Bedrock, Google Vertex AI, or Azure OpenAI.

### Supported on `dev/rust` (this branch)

| Provider | Protocol | Auth env var(s) | Base URL env var | Default base URL |
|---|---|---|---|---|
| **Anthropic** (direct) | Anthropic Messages API | `ANTHROPIC_API_KEY` or `ANTHROPIC_AUTH_TOKEN` or OAuth (`claw login`) | `ANTHROPIC_BASE_URL` | `https://api.anthropic.com` |

That's it. On `dev/rust`, the `api` crate has a single provider
backend (`rust/crates/api/src/client.rs`) wired directly to
Anthropic's Messages API. There is no `providers/` module, no
auto-routing by model prefix, and no OpenAI-compatible adapter. If you
export `OPENAI_API_KEY`, `XAI_API_KEY`, or `DASHSCOPE_API_KEY` on this
branch, claw will ignore them and still fail with `MissingApiKey`
because it only looks at `ANTHROPIC_*`.

### Additionally supported on `main`

`main` has a multi-provider routing layer under
`rust/crates/api/src/providers/` that `dev/rust` does not yet carry.
If you need any of these, build from `main` and wait for the routing
work to land on `dev/rust`:

| Provider | Protocol | Auth env var | Default base URL |
|---|---|---|---|
| **xAI** (Grok) | OpenAI-compatible | `XAI_API_KEY` | `https://api.x.ai/v1` |
| **OpenAI** | OpenAI Chat Completions | `OPENAI_API_KEY` | `https://api.openai.com/v1` |
| **DashScope** (Alibaba Qwen) | OpenAI-compatible | `DASHSCOPE_API_KEY` | `https://dashscope.aliyuncs.com/compatible-mode/v1` |

Any service that speaks the OpenAI `/v1/chat/completions` wire format
also works by pointing `OPENAI_BASE_URL` at it (OpenRouter, Ollama,
local LLM proxies, etc.).

On `main`, the provider is selected automatically by model-name prefix
(`claude` → Anthropic, `grok` → xAI, `openai/` or `gpt-` → OpenAI,
`qwen/` or `qwen-` → DashScope) before falling through to whichever
credential is present. Prefix routing wins over env-var presence, so
setting `ANTHROPIC_API_KEY` will not silently hijack an
`openai/gpt-4.1-mini` request.

### Not supported anywhere in this repo (yet)

These are the provider backends people reasonably expect to work but
which **do not have any code path** in either `dev/rust` or `main` as
of this commit. Setting the corresponding cloud SDK env vars will not
make them work — there is nothing to wire them into.

| Provider | Why it doesn't work today |
|---|---|
| **AWS Bedrock** | No SigV4 signer, no Bedrock-specific request adapter, no `AWS_*` credential path in the api crate. Bedrock's Claude endpoint is wire-compatible with a different request envelope than direct Anthropic and would need a dedicated backend. |
| **Google Vertex AI (Anthropic on Vertex)** | No Google auth (service account / ADC) path, no Vertex-specific base URL adapter. Vertex publishes Claude models under a `projects/<proj>/locations/<loc>/publishers/anthropic/models/<model>:streamRawPredict` URL shape that requires a separate route. |
| **Azure OpenAI** | OpenAI wire format but uses `api-version` query params, `api-key` header (not `Authorization: Bearer`), and deployment-name routing instead of model IDs. The `main`-branch OpenAI-compatible adapter assumes upstream OpenAI semantics and won't round-trip Azure deployments cleanly. |
| **Google AI Studio (Gemini)** | Different request shape entirely; not OpenAI-compatible and not Anthropic-compatible. Would need its own backend. |

If you need one of these, the honest answer today is: use a proxy that
speaks Anthropic or OpenAI on its public side and translates to
Bedrock/Vertex/Azure/Gemini internally. Setting `ANTHROPIC_BASE_URL`
or (on `main`) `OPENAI_BASE_URL` at a translation proxy is the
supported escape hatch until first-class backends land.

### When auth fails

If you see `ANTHROPIC_AUTH_TOKEN or ANTHROPIC_API_KEY is not set` on
`dev/rust` after setting, say, `OPENAI_API_KEY`, that's not a bug —
it's this branch telling you honestly that it doesn't yet know how to
talk to OpenAI. Either build from `main`, or export
`ANTHROPIC_API_KEY`, or run `claw login` to use Anthropic OAuth.

## Features

| Feature | Status |
|---------|--------|
| Anthropic API + streaming | ✅ |
| OAuth login/logout | ✅ |
| Interactive REPL (rustyline) | ✅ |
| Tool system (bash, read, write, edit, grep, glob) | ✅ |
| Web tools (search, fetch) | ✅ |
| Sub-agent orchestration | ✅ |
| Todo tracking | ✅ |
| Notebook editing | ✅ |
| CLAUDE.md / project memory | ✅ |
| Config file hierarchy (.claude.json) | ✅ |
| Permission system | ✅ |
| MCP server lifecycle | ✅ |
| Session persistence + resume | ✅ |
| Extended thinking (thinking blocks) | ✅ |
| Cost tracking + usage display | ✅ |
| Git integration | ✅ |
| Markdown terminal rendering (ANSI) | ✅ |
| Model aliases (opus/sonnet/haiku) | ✅ |
| Slash commands (/status, /compact, /clear, etc.) | ✅ |
| Hooks (PreToolUse/PostToolUse) | 🔧 Config only |
| Plugin system | 📋 Planned |
| Skills registry | 📋 Planned |

## Model Aliases

Short names resolve to the latest model versions:

| Alias | Resolves To |
|-------|------------|
| `opus` | `claude-opus-4-6` |
| `sonnet` | `claude-sonnet-4-6` |
| `haiku` | `claude-haiku-4-5-20251213` |

## CLI Flags

```
claw [OPTIONS] [COMMAND]

Options:
  --model MODEL                    Set the model (alias or full name)
  --dangerously-skip-permissions   Skip all permission checks
  --permission-mode MODE           Set read-only, workspace-write, or danger-full-access
  --allowedTools TOOLS             Restrict enabled tools
  --output-format FORMAT           Output format (text or json)
  --version, -V                    Print version info

Commands:
  prompt <text>      One-shot prompt (non-interactive)
  login              Authenticate via OAuth
  logout             Clear stored credentials
  init               Initialize project config
  doctor             Check environment health
  self-update        Update to latest version
```

## Slash Commands (REPL)

| Command | Description |
|---------|-------------|
| `/help` | Show help |
| `/status` | Show session status (model, tokens, cost) |
| `/cost` | Show cost breakdown |
| `/compact` | Compact conversation history |
| `/clear` | Clear conversation |
| `/model [name]` | Show or switch model |
| `/permissions` | Show or switch permission mode |
| `/config [section]` | Show config (env, hooks, model) |
| `/memory` | Show CLAUDE.md contents |
| `/diff` | Show git diff |
| `/export [path]` | Export conversation |
| `/session [id]` | Resume a previous session |
| `/version` | Show version |

## Workspace Layout

```
rust/
├── Cargo.toml              # Workspace root
├── Cargo.lock
└── crates/
    ├── api/                # Anthropic API client + SSE streaming
    ├── commands/           # Shared slash-command registry
    ├── compat-harness/     # TS manifest extraction harness
    ├── runtime/            # Session, config, permissions, MCP, prompts
    ├── rusty-claude-cli/   # Main CLI binary (`claw`)
    └── tools/              # Built-in tool implementations
```

### Crate Responsibilities

- **api** — HTTP client, SSE stream parser, request/response types, auth (API key + OAuth bearer)
- **commands** — Slash command definitions and help text generation
- **compat-harness** — Extracts tool/prompt manifests from upstream TS source
- **runtime** — `ConversationRuntime` agentic loop, `ConfigLoader` hierarchy, `Session` persistence, permission policy, MCP client, system prompt assembly, usage tracking
- **rusty-claude-cli** — REPL, one-shot prompt, streaming display, tool call rendering, CLI argument parsing
- **tools** — Tool specs + execution: Bash, ReadFile, WriteFile, EditFile, GlobSearch, GrepSearch, WebSearch, WebFetch, Agent, TodoWrite, NotebookEdit, Skill, ToolSearch, REPL runtimes

## Stats

- **~20K lines** of Rust
- **6 crates** in workspace
- **Binary name:** `claw`
- **Default model:** `claude-opus-4-6`
- **Default permissions:** `danger-full-access`

## License

See repository root.
