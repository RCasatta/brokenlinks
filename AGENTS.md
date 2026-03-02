# Agents Rules for brokenlinks

## Build & Commands

```bash
# Build project
cargo build

# Run tests (single test)
cargo test --test <test_name>
cargo test --bin brokenlinks <filter>

# Format code
cargo fmt

# Lint/check code
cargo clippy -- -D warnings
```

## Project Overview

Simple CLI tool to find broken links in a website. Uses reqwest for HTTP requests, select for HTML parsing, and threadpool for concurrent fetching.

## Code Style Guidelines

### Imports & Module Organization
- Standard library imports first (std::...)
- External crates second (reqwest::..., clap::...)
- Project-specific modules third
- Use `use` statements for all types; avoid `self.*` qualification unless necessary
- Group related imports together with blank lines between groups

### Formatting
- Follow Rust standard formatting (`cargo fmt`)
- Maximum line length: 100 characters
- Indentation: 4 spaces, no tabs

### Naming Conventions
- Functions: snake_case (e.g., `get_url_and_extract`, `validate_and_make_full_url`)
- Types/structs: PascalCase (e.g., `Args`)
- Constants: UPPER_SNAKE_CASE
- Variables: snake_case, use descriptive names
- Module names: lowercase

### Error Handling
- Use `Box<dyn Error>` for top-level error returns in simple binaries
- Propagate errors with `?` operator where possible
- Avoid `unwrap()` or `expect()` except in initialization code
- Handle specific errors before falling back to generic cases
- Log errors meaningfully with context (e.g., `"KO {} {}", url, e`)

### Type System
- Prefer concrete types over `dyn Trait` when possible
- Use `Option<T>` for potentially missing values
- Use `Result<T, E>` for fallible operations
- Leverage Rust's type inference (`let x = ...;`) where clear

### Request Configuration
- Always configure reqwest client with compression features enabled:
  ```rust
  Client::builder()
      .gzip(true)
      .deflate(true)
      .brotli(true)
      .build()?
  ```
- Add compression features to Cargo.toml: `features = ["blocking", "gzip", "deflate", "brotli"]`

### Concurrency
- Use `ThreadPool` for concurrent HTTP requests
- Share clients via `.clone()` (cheap cloneable)
- Use channels (`std::sync::mpsc`) for URL distribution
- Avoid shared mutable state; prefer message passing

### HTTP Handling
- Always check response status before processing body
- Handle HEAD failures gracefully by falling back to GET
- Check `CONTENT_TYPE` header before parsing HTML
- Remove URL fragments before storing/comparing URLs
- Validate relative URLs against base URL using `Url::join()`

### Testing Strategy
- Integration tests preferred for end-to-end behavior
- Test URL validation and normalization separately
- Mock HTTP responses where possible
- Test edge cases: malformed URLs, non-HTML content, redirects

## Known Patterns

### Client Creation Pattern
```rust
fn create_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .gzip(true)
        .deflate(true)
        .brotli(true)
        .timeout(Duration::from_secs(30))
        .build()
}
```

### URL Processing Pattern
```rust
let mut url = base.join(path)?;
url.set_fragment(None); // Always strip fragments
if url.host() == base.host() { /* internal link */ }
```

## Dependencies

- `reqwest` - HTTP client with blocking support and compression
- `clap` - CLI argument parsing
- `select` - HTML/CSS selector-based parsing
- `threadpool` - Concurrent task execution
- `url` - URL parsing and manipulation