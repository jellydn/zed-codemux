# Directory Structure

**Analysis Date:** 2026-07-23

## Top-Level Layout

```
.
├── Cargo.toml              # Root package manifest, workspace definition
├── Cargo.lock              # Locked dependency versions
├── justfile                # Task runner definitions
├── prek.toml               # Pre-commit hook configuration
├── renovate.json           # Dependency update automation
├── LICENSE                 # MIT License
├── README.md               # User-facing documentation
├── AGENTS.md               # Agent/AI tooling notes
├── CLAUDE.md               # Claude-specific guidance
├── US_VERIFICATION.md      # User story verification checklist
├── plan.md                 # Project plan
├── autoresearch.ideas.md   # Auto-research notes
├── autoresearch.jsonl      # Auto-research data
│
├── src/                    # Main binary source
│   ├── main.rs             # Entry point, CLI, MuxLauncher trait, exec dispatch
│   ├── config.rs           # TOML config loader + parser
│   ├── detect.rs           # Multiplexer detection (env → config → PATH)
│   ├── sanitize.rs         # Session name sanitization + unique name generator
│   ├── tmux.rs             # Tmux multiplexer launcher
│   ├── zellij.rs           # Zellij multiplexer launcher
│   ├── upgrade.rs          # Self-upgrade (GitHub API, download, atomic replace)
│   ├── main_tests.rs       # Tests for main module (session naming, shell escape)
│   ├── config_tests.rs     # Tests for config parsing
│   ├── detect_tests.rs     # Tests for multiplexer detection
│   ├── sanitize_tests.rs   # Tests for session name sanitization
│   ├── tmux_tests.rs       # Tests for tmux command building
│   └── zellij_tests.rs     # Tests for zellij command building
│
├── tests/                  # Integration tests
│   └── cli.rs              # End-to-end CLI tests (--version, --help, --init)
│
├── extension/              # Zed extension crate (workspace member)
│   ├── Cargo.toml          # cdylib crate manifest
│   ├── extension.toml      # Zed extension metadata
│   └── src/
│       └── lib.rs          # Extension entry point (discoverability)
│
├── docs/                   # Supplementary documentation
│   └── zed-integration.md  # Zed-specific setup guide
│
├── .github/workflows/      # CI/CD pipelines
│   ├── ci.yml              # Build + test on push/PR
│   └── release.yml         # Build, publish, release on tag
│
├── Formula/                # Homebrew formula (for reference)
│   └── codemux.rb
│
├── goals/                  # Feature goal/planning documents
│   └── upgrade/            # Upgrade feature plan
│
├── .planning/codebase/     # Generated codebase documentation (codemap)
│   ├── STACK.md
│   ├── INTEGRATIONS.md
│   ├── ARCHITECTURE.md
│   ├── STRUCTURE.md        # This file
│   ├── CONVENTIONS.md
│   ├── TESTING.md
│   └── CONCERNS.md
│
├── scripts/                # Utility scripts
│   ├── setup-secrets.sh    # GitHub secrets bootstrap
│   └── ralph/              # Autonomous agent loop
│       ├── ralph.sh
│       ├── prd.json
│       └── src/main.rs
│
└── tasks/                  # Task/PRD documents
    └── prd-zed-mux.md
```

## Naming Conventions

| Category | Convention | Examples |
|----------|-----------|----------|
| Source files | `snake_case.rs` | `config.rs`, `sanitize.rs`, `upgrade.rs` |
| Test files | `<module>_tests.rs` | `config_tests.rs`, `main_tests.rs` |
| Binary name | `snake_case` | `codemux` |
| Extension crate | `kebab-case` | `codemux-extension` |
| Scripts | `kebab-case.sh` | `setup-secrets.sh` |

## Key Locations

| What | Where |
|------|-------|
| Entry point | `src/main.rs::main()` |
| CLI flag parsing | `src/main.rs::parse_args()` |
| Config file (user) | `~/.config/codemux/config.toml` |
| Default config content | `src/config.rs::DEFAULT_CONFIG_CONTENT` |
| MuxLauncher trait | `src/main.rs` (trait definition) |
| Session sanitization | `src/sanitize.rs` |
| Multiplexer detection | `src/detect.rs` |
| Shell escaping | `src/main.rs::shell_escape()` |
| Upgrade logic | `src/upgrade.rs::upgrade()` |
| Zed extension entry | `extension/src/lib.rs` |
