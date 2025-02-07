---

#   Current layout

```
Cargo.toml
src/
*   scan/
    *   impl1
    *   ...
    scan.rs
    main.rs
    support.rs
    exit.rs
```


#   New layout

```
Cargo.toml              // Workspace
deps/
*   rust-gpu/           // Git
    *   Cargo.toml      // Workspace
*   support/            // In-repo
    *   Cargo.toml      // Package
    *   src/
        *   lib.rs
impls/
*   impl1/
    *   Cargo.toml      // Package
    *   src/
        *   lib.rs
        *   scan.rs
*   ...
src/
*   main.rs             // `scan` binary
tests/
*   test-scan.rs        // Run `scan` binary
```
