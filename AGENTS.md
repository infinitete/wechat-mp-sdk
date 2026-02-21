# AGENTS.md - AI Coding Agent Guidelines

## Project Overview

Rust library crate (`applet`) using edition 2021, part of the WxSdk workspace.

---

## Build, Lint, and Test Commands

### Building
```bash
cargo check          # Fast error check (no binary)
cargo build          # Debug build
cargo build --release  # Optimized release
cargo doc --open     # Build & open docs
```

### Linting (MUST pass before commit)
```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check    # Format check
cargo fmt            # Auto-format
```

### Testing
```bash
cargo test                    # All tests
cargo test -- --nocapture     # With output
cargo test test_name          # Single test (partial match)
cargo test it_works           # Example: run specific test
cargo test tests::            # Tests in module
cargo test -- --ignored       # Run ignored tests
cargo test --doc              # Doc tests only
```

### Dependencies
```bash
cargo add <crate>             # Add dependency
cargo add --dev <crate>       # Dev dependency
cargo update                  # Update deps
```

---

## Code Style Guidelines

### Formatting
- `cargo fmt` before every commit - NO exceptions
- Max line width: 100 chars | 4 spaces | K&R braces

### Imports (group with blank lines)
```rust
// 1. Standard library
use std::collections::HashMap;
// 2. External crates
use serde::{Deserialize, Serialize};
// 3. Current crate
use crate::config::Settings;
// 4. Parent module
use super::utils::helper;
```

### Naming Conventions
| Type | Convention | Example |
|------|------------|---------|
| Crates/Modules | snake_case | `wx_sdk`, `auth_handler` |
| Types/Traits | PascalCase | `UserProfile`, `FromStr` |
| Functions/Variables | snake_case | `get_user_by_id()` |
| Constants/Static | SCREAMING_SNAKE_CASE | `MAX_RETRIES` |
| Lifetimes | lowercase | `'a`, `'static` |
| Type params | uppercase | `T`, `E`, `K` |

### Error Handling
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Use ? for propagation
pub fn process_file(path: &Path) -> Result<String, AppError> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

// AVOID unwrap()/expect() in library code (OK in tests only)
```

### Documentation
```rust
/// Brief description.
///
/// # Arguments
/// * `name` - Parameter description
///
/// # Returns / # Errors / # Examples
pub fn function_name(name: &str) -> Result<(), AppError> { }
```

### Type Safety
```rust
// PREFERRED: Strong types / newtype pattern
pub struct UserId(String);

#[derive(Debug, Clone, PartialEq)]
pub struct Percentage(u8);

impl Percentage {
    pub fn new(value: u8) -> Result<Self, AppError> {
        if value > 100 { return Err(AppError::InvalidPercentage(value)); }
        Ok(Self(value))
    }
}

// NEVER suppress type errors
```

### Module Organization
```
src/
├── lib.rs      # Re-export public API
├── error.rs    # Error types
└── api/
    ├── mod.rs
    ├── user.rs
    └── auth.rs
```

```rust
// src/lib.rs
mod error;
pub use error::AppError;
pub mod api;
```

### Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_describes_expected_behavior() {
        let result = process("input");
        assert!(result.is_ok());
    }

    #[test]
    fn test_with_result() -> Result<(), Box<dyn std::error::Error>> {
        let value = parse("valid")?;
        assert_eq!(value, 42);
        Ok(())
    }
}
```

---

## Git Hooks

This project uses git hooks to ensure code quality before each commit.

### Setup
Run this once after cloning:
```bash
./setup-hooks.sh
```

### What the hooks check
- `cargo fmt --check` - Code formatting
- `cargo clippy` - Linting (warnings as errors)
- `cargo test` - Unit tests

### Bypass hooks (not recommended)
```bash
git commit --no-verify
```

---

## Pre-Commit Checklist
1. `cargo fmt` - Format code
2. `cargo clippy -- -D warnings` - No warnings
3. `cargo test` - All tests pass
4. `cargo doc` - Docs build cleanly

---

## Common Dependencies
- `thiserror` - Custom error types
- `serde` - Serialization
- `tokio` - Async runtime

---

## File Structure
```
applet/
├── Cargo.toml
├── src/
│   └── lib.rs
├── tests/        # Integration tests
└── examples/     # Examples
```
