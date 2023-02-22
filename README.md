# Crusty Rustacean API

This is the repo in support of my effort to build an API in support of my blog, [Crusty Rustacean](https://crusty-rustacean-dev.shuttleapp.rs). This readme will be a diary of sorts as I work through the book to document and reflect on what I learn along the way.

## 2023-02-21
- project skeleton started and pushed up to a fresh GitHub repo: [crusty-rustacean-api](https://github.com/crusty-rustacean-api)

- toolchain setup complete
  - VS Code with Rust Analyzer

- Inner Development Loop
  - lld linker installed
  - cargo watch installed

- Continuous Integration (CI)
  - tests: cargo test
  - code coverage: cargo tarpaulin
  - linting: cargo clippy
    - cargo clippy -- -D warnings (fails the linter check if clippy emits any warnings)
  - formatting: cargo fmt
    - fine tune formatting for any project with a rustfmt.toml configuration file
  - auditing for security vulnerabilities: cargo audit
  - CI pipeline
    - installed GitHub Actions (per the book examples) in my repo
      - Actions seem to have lots of issues and warnings about things being deprecated, will have to explore this further

