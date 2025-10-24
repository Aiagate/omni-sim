# Development Guidelines (Rust)

This document contains critical information about working with this Rust codebase. Follow these guidelines precisely.

## Core Development Rules

1. Package Management
   - ONLY use cargo, NEVER manual downloads
   - Adding dependencies: `cargo add package`
   - Running tools: `cargo run`
   - Building: `cargo build` or `cargo build --release`
   - Updating: `cargo update`
   - FORBIDDEN: Direct modification of Cargo.toml versions

2. Code Quality
   - Use idiomatic Rust patterns
   - Prefer ownership over cloning when possible
   - Error handling with Result/Option (no unwrap in production)
   - Public APIs must have doc comments (`///`)
   - Functions must be focused and small
   - Follow existing patterns exactly
   - Line length: 100 chars maximum (rustfmt default)

3. Testing Requirements
   - Framework: `cargo test`
   - Unit tests in same file as code (mod tests)
   - Integration tests in tests/ directory
   - Coverage: test edge cases and errors
   - New features require tests
   - Bug fixes require regression tests
   - Async testing: use tokio::test or async-std

- For commits fixing bugs or adding features based on user reports add:
  ```bash
  git commit --trailer "Reported-by:<name>"
  ```
  Where `<name>` is the name of the user.

- For commits related to a Github issue, add
  ```bash
  git commit --trailer "Github-Issue:#<number>"
  ```
- NEVER ever mention a `co-authored-by` or similar aspects. In particular, never
  mention the tool used to create the commit message or PR.

## Pull Requests

- Create a detailed message of what changed. Focus on the high level description of
  the problem it tries to solve, and how it is solved. Don't go into the specifics of the
  code unless it adds clarity.

- Always add `jerome3o-anthropic` and `jspahrsummers` as reviewer.

- NEVER ever mention a `co-authored-by` or similar aspects. In particular, never
  mention the tool used to create the commit message or PR.

## Rust Tools

## Code Formatting

1. Rustfmt
   - Format: `cargo fmt`
   - Check: `cargo fmt -- --check`
   - Config: rustfmt.toml or .rustfmt.toml
   - Critical issues:
     - Line length (100 chars)
     - Import grouping
     - Consistent spacing
   - Line wrapping:
     - Strings: use format! or raw strings
     - Function calls: multi-line with proper indent
     - Imports: rustfmt handles automatically

2. Clippy (Linting)
   - Tool: `cargo clippy`
   - Strict mode: `cargo clippy -- -D warnings`
   - Requirements:
     - Fix all warnings
     - No use of unwrap() in production code
     - Proper error propagation with ?
     - Avoid unnecessary clones
   - Common lints:
     - clippy::unwrap_used
     - clippy::expect_used
     - clippy::panic
     - clippy::todo

3. Type Checking
   - Built into cargo: `cargo check`
   - Faster than full build
   - Checks for type errors and basic issues

4. Documentation
   - Generate: `cargo doc --open`
   - Doc tests: run with `cargo test --doc`
   - All public items need `///` doc comments
   - Include examples in doc comments

5. Pre-commit
   - Config: `.pre-commit-config.yaml`
   - Runs: on git commit
   - Tools: Prettier (YAML/JSON), rustfmt, clippy
   - Updates:
     - Check crates.io versions
     - Update config
     - Commit config first

## Error Resolution

1. CI Failures
   - Fix order:
     1. Formatting (`cargo fmt`)
     2. Type errors (`cargo check`)
     3. Linting (`cargo clippy`)
     4. Tests (`cargo test`)
   - Type errors:
     - Read error message carefully
     - Check lifetime annotations
     - Verify trait bounds
     - Check ownership/borrowing

2. Common Issues
   - Line length:
     - Let rustfmt handle it
     - Break long chains with intermediate variables
   - Borrowing:
     - Understand ownership rules
     - Use references when possible
     - Clone only when necessary
   - Lifetimes:
     - Let compiler infer when possible
     - Explicit annotations only when needed
   - Async:
     - Use .await consistently
     - Handle Send/Sync bounds
     - Match async runtime (tokio/async-std)

3. Best Practices
   - Check git status before commits
   - Run formatters before type checks
   - Keep changes minimal
   - Follow existing patterns
   - Document public APIs
   - Test thoroughly
   - Use cargo clippy to catch common mistakes
   - Profile before optimizing
   - Prefer iterators over loops
   - Use type system for correctness
