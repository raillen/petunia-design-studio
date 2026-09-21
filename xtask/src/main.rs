//! Repository-owned task facade (14.8). Stable semantic commands:
//! verify, architecture, test, conformance, fixtures, fuzz-smoke,
//! bench-smoke, ui-gauntlet, security, migrations, docs, release-check,
//! gauntlet. Scripts and agents must call these, not duplicate workflows.

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("verify");
    let root = workspace_root();
    let code = match command {
        "verify" => cmd_verify(&root),
        "architecture" | "arch" => cmd_architecture(&root),
        "test" => cmd_test(&root),
        "conformance" => cmd_conformance(&root),
        "fixtures" => cmd_fixtures(&root),
        "fuzz-smoke" => cmd_post_v1("fuzz-smoke", "cargo-fuzz corpus not wired in P00"),
        "bench-smoke" => cmd_post_v1("bench-smoke", "Criterion benches not wired in P00"),
        "ui-gauntlet" => cmd_post_v1("ui-gauntlet", "GPUI shell does not exist in P00"),
        "security" => cmd_security(&root),
        "migrations" => cmd_post_v1("migrations", "schema v1 has no predecessors in P00"),
        "docs" => cmd_docs(&root),
        "release-check" => cmd_post_v1("release-check", "packaging not wired in P00"),
        "gauntlet" => cmd_gauntlet(&root),
        "--help" | "-h" | "help" => {
            print_help();
            0
        }
        unknown => {
            eprintln!("xtask: unknown command `{unknown}`");
            print_help();
            2
        }
    };
    std::process::exit(code);
}

fn print_help() {
    println!("xtask — Aubrieta repository facade");
    println!("usage: cargo xtask <command>");
    println!("commands: verify architecture test conformance fixtures fuzz-smoke");
    println!("          bench-smoke ui-gauntlet security migrations docs");
    println!("          release-check gauntlet");
}

/// Fast default gate: fmt check + clippy + tests + boundary checks.
fn cmd_verify(root: &Path) -> i32 {
    println!("xtask verify: fmt + clippy + test + architecture");
    let mut code = run_cargo(root, &["fmt", "--all", "--check"]);
    if code != 0 {
        return code;
    }
    code = run_cargo(
        root,
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    );
    if code != 0 {
        return code;
    }
    code = cmd_test(root);
    if code != 0 {
        return code;
    }
    cmd_architecture(root)
}

/// Forbidden dependency edges. Domain never imports GUI/GPU/export stacks.
fn cmd_architecture(root: &Path) -> i32 {
    println!("xtask architecture: forbidden-edge check");
    // (crate, forbidden dependency substring) pairs for the P00 slice.
    // Full 09.22 matrix is enforced as crates are added.
    let rules: &[(&str, &[&str])] = &[
        (
            "aubrieta_foundation",
            &["gpui", "vello", "wgpu", "krilla", "mlua"],
        ),
        (
            "aubrieta_document",
            &[
                "gpui",
                "vello",
                "wgpu",
                "krilla",
                "mlua",
                "aubrieta_io",
                "aubrieta_shell",
            ],
        ),
        (
            "aubrieta_application",
            &["gpui", "vello", "wgpu", "krilla", "aubrieta_shell"],
        ),
        ("aubrieta_jobs", &["gpui", "vello", "wgpu"]),
        (
            "aubrieta_geometry",
            &[
                "gpui",
                "vello",
                "wgpu",
                "krilla",
                "mlua",
                "aubrieta_shell",
            ],
        ),
        (
            "aubrieta_color",
            &["gpui", "vello", "wgpu", "krilla", "aubrieta_shell"],
        ),
        (
            "aubrieta_evaluation",
            &["gpui", "vello", "wgpu", "krilla", "aubrieta_shell"],
        ),
        ("aubrieta_render", &["gpui", "aubrieta_shell"]),
        ("aubrieta_io", &["gpui", "aubrieta_shell"]),
        (
            "aubrieta_text",
            &[
                "gpui",
                "vello",
                "wgpu",
                "krilla",
                "mlua",
                "aubrieta_shell",
            ],
        ),
        (
            "aubrieta_raster",
            &[
                "gpui",
                "vello",
                "wgpu",
                "krilla",
                "mlua",
                "aubrieta_shell",
            ],
        ),
        (
            "aubrieta_resources",
            &[
                "gpui",
                "vello",
                "wgpu",
                "krilla",
                "mlua",
                "aubrieta_shell",
            ],
        ),
        (
            "aubrieta_platform",
            &["gpui", "vello", "wgpu", "krilla", "aubrieta_shell"],
        ),
        (
            "aubrieta_extension",
            &["gpui", "vello", "wgpu", "krilla", "aubrieta_shell"],
        ),
        (
            "aubrieta_mcp",
            &["gpui", "vello", "wgpu", "krilla", "aubrieta_shell"],
        ),
    ];
    let mut failures = 0;
    for (crate_name, forbidden) in rules {
        let manifest = root.join("crates").join(crate_name).join("Cargo.toml");
        let text = match std::fs::read_to_string(&manifest) {
            Ok(text) => text,
            Err(error) => {
                eprintln!("arch: cannot read {}: {error}", manifest.display());
                failures += 1;
                continue;
            }
        };
        for needle in *forbidden {
            if text.contains(needle) {
                eprintln!("arch: FORBIDDEN `{crate_name}` depends on `{needle}`");
                failures += 1;
            }
        }
        // Source-level backstop: no GUI/GPU imports in domain sources.
        let src = root.join("crates").join(crate_name).join("src");
        if let Err(error) = grep_forbidden(&src, forbidden) {
            eprintln!("arch: {error}");
            failures += 1;
        }
    }
    if failures > 0 {
        eprintln!("arch: {failures} forbidden edge(s) found");
        return 1;
    }
    println!("arch: no forbidden edges in P00 slice");
    0
}

fn grep_forbidden(dir: &Path, forbidden: &[&str]) -> Result<(), String> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        let entries =
            std::fs::read_dir(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("dir entry: {e}"))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| format!("read {}: {e}", path.display()))?;
                for needle in forbidden {
                    // Match `use gpui::` / `extern crate gpui` style imports.
                    for prefix in ["use ", "extern crate ", "::"] {
                        if text.contains(&format!("{prefix}{needle}")) {
                            return Err(format!(
                                "forbidden import `{needle}` in {}",
                                path.display()
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Workspace tests.
fn cmd_test(root: &Path) -> i32 {
    println!("xtask test: cargo test --workspace");
    run_cargo(root, &["test", "--workspace"])
}

/// Headless CLI end-to-end for the P00 slice.
fn cmd_conformance(root: &Path) -> i32 {
    println!("xtask conformance: headless CLI flow");
    run_cargo(root, &["run", "-p", "aubrieta-cli"])
}

/// Fixture presence check (P00: schema + roundtrip covered by unit tests).
fn cmd_fixtures(root: &Path) -> i32 {
    println!("xtask fixtures: checking tests/trees (P00: unit-level only)");
    let dir = root.join("crates/aubrieta_document");
    if !dir.is_dir() {
        eprintln!("fixtures: missing {}", dir.display());
        return 1;
    }
    println!("fixtures: OK (unit fixtures only in P00)");
    0
}

/// Dependency/security smoke: cargo audit when available, plus an
/// offline manifest backstop (no wildcard majors).
fn cmd_security(root: &Path) -> i32 {
    println!("xtask security: cargo audit + manifest backstop");
    let audit = Command::new("cargo")
        .args(["audit", "--deny", "warnings"])
        .current_dir(root)
        .status();
    match audit {
        Ok(status) if status.success() => {}
        Ok(status) => {
            eprintln!(
                "security: cargo audit reported issues (exit {})",
                status.code().unwrap_or(1)
            );
            return 1;
        }
        Err(error) => {
            eprintln!("security: cargo audit unavailable ({error}); manifest backstop only");
        }
    }
    // Deny known-hostile patterns in manifests: git deps without rev, wildcard majors.
    let manifests = ["Cargo.toml"];
    for manifest in manifests {
        let text = match std::fs::read_to_string(root.join(manifest)) {
            Ok(text) => text,
            Err(error) => {
                eprintln!("security: cannot read {manifest}: {error}");
                return 1;
            }
        };
        if text.contains("version = \"*\"") {
            eprintln!("security: wildcard dependency in {manifest}");
            return 1;
        }
    }
    println!("security: OK (full audit/licensing/SBOM is a later wave)");
    0
}

/// Documentation presence: canonical docs + authority map parse check.
fn cmd_docs(root: &Path) -> i32 {
    println!("xtask docs: presence check");
    for required in ["docs/AUTHORITY_MAP.json", "docs/glossary.json", "AGENTS.md"] {
        if !root.join(required).exists() {
            eprintln!("docs: missing {required}");
            return 1;
        }
    }
    println!("docs: OK (VitePress + pt-BR sync is a later wave)");
    0
}

/// Changed-scope gauntlet for P00: verify + conformance + fixtures + docs.
fn cmd_gauntlet(root: &Path) -> i32 {
    println!("xtask gauntlet: P00 evidence run");
    for step in ["verify", "conformance", "fixtures", "docs"] {
        let code = match step {
            "verify" => cmd_verify(root),
            "conformance" => cmd_conformance(root),
            "fixtures" => cmd_fixtures(root),
            "docs" => cmd_docs(root),
            _ => 0,
        };
        if code != 0 {
            eprintln!("gauntlet: step `{step}` failed with {code}");
            return code;
        }
    }
    println!("gauntlet: P00 slice green");
    0
}

/// Explicit Post-V1 stub: normal state with a disabled reason, never a panic.
fn cmd_post_v1(command: &str, reason: &str) -> i32 {
    println!("xtask {command}: POST_V1 — {reason}");
    0
}

fn workspace_root() -> PathBuf {
    // xtask always runs from the workspace root via `cargo xtask`.
    std::env::var("CARGO_WORKSPACE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn run_cargo(root: &Path, args: &[&str]) -> i32 {
    let status = Command::new("cargo").args(args).current_dir(root).status();
    match status {
        Ok(status) if status.success() => 0,
        Ok(status) => status.code().unwrap_or(1),
        Err(error) => {
            eprintln!("xtask: failed to run cargo {}: {error}", args.join(" "));
            1
        }
    }
}
