// src/server/mod.rs
// ═══════════════════════════════════════════════════════
// Tiny Axum web server
//   GET  /    → static/index.html
//   POST /run → multipart form → full pipeline → NDJSON
// ═══════════════════════════════════════════════════════

use axum::{
    extract::Multipart,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use anyhow::Result;

use crate::types::{AnalysisConfig, InputBound, Verdict};

// ── Launch ────────────────────────────────────────────

pub async fn launch(port: u16) -> Result<()> {
    let app = Router::new()
        .route("/",    get(serve_ui))
        .route("/run", post(run_check));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("\n  ┌─────────────────────────────────────────┐");
    println!("  │  EQ·CHECK UI  →  http://localhost:{}   │", port);
    println!("  │  Press Ctrl+C to stop                   │");
    println!("  └─────────────────────────────────────────┘\n");

    let url = format!("http://localhost:{}", port);
    let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    let _ = std::process::Command::new("open").arg(&url).spawn();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// ── GET / ─────────────────────────────────────────────

async fn serve_ui() -> impl IntoResponse {
    let html = std::fs::read_to_string("static/index.html").unwrap_or_else(|_| {
        "<h1>UI not found</h1><p>Place <code>static/index.html</code> in project root.</p>"
            .to_string()
    });
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"));
    (headers, html)
}

// ── Wire types ────────────────────────────────────────

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Msg {
    Log    { level: String, text: String },
    Result {
        equivalent:     bool,
        paths_c:        usize,
        paths_rust:     usize,
        inputs_tested:  usize,
        counterexample: Option<CeMsg>,
        /// Semantic diff — None when programs are equivalent
        diff:           Option<SemanticDiffMsg>,
        time_taken:     f64,
    },
    Error  { text: String },
}

#[derive(Serialize, Clone)]
pub struct CeMsg {
    pub inputs:   Vec<(String, i64)>,
    pub c_return: String,
    pub r_return: String,
}

/// What the browser needs to render the semantic diff panel.
/// All data here comes from KLEE constraints, NOT text comparison.
#[derive(Serialize, Clone)]
pub struct SemanticDiffMsg {
    // ── Human-readable divergent condition ────────────
    /// e.g.  "x <= 10"
    pub condition_c:    String,
    /// e.g.  "x < 10"
    pub condition_rust: String,

    // ── Source location (1-based line numbers) ────────
    pub diverge_line_c:    Option<usize>,
    pub diverge_line_rust: Option<usize>,

    // ── Source windows (±5 lines around divergence) ───
    pub c_lines:    Vec<DiffLine>,
    pub rust_lines: Vec<DiffLine>,

    // ── Fix suggestion ────────────────────────────────
    pub suggestion: String,

    // ── Informational ─────────────────────────────────
    /// Extra paths Rust has vs C (safety check branches)
    pub extra_rust_paths: usize,
}

#[derive(Serialize, Clone)]
pub struct DiffLine {
    pub num:       usize,
    pub text:      String,
    /// true = this is the semantically divergent line
    pub highlight: bool,
}

// ── POST /run ─────────────────────────────────────────

async fn run_check(mut multipart: Multipart) -> impl IntoResponse {
    let mut c_name    = String::new();
    let mut r_name    = String::new();
    let mut c_bytes   = Vec::<u8>::new();
    let mut r_bytes   = Vec::<u8>::new();
    let mut function  = String::new();
    let mut bounds    = String::from("x:0:100");
    let mut timeout   = 60u32;
    let mut max_paths = 100u32;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "c_file"    => { c_name  = field.file_name().unwrap_or("prog.c").to_string();
                             c_bytes = field.bytes().await.unwrap_or_default().to_vec(); }
            "rust_file" => { r_name  = field.file_name().unwrap_or("prog.rs").to_string();
                             r_bytes = field.bytes().await.unwrap_or_default().to_vec(); }
            "function"  => { function  = field.text().await.unwrap_or_default(); }
            "bounds"    => { bounds    = field.text().await.unwrap_or(bounds); }
            "timeout"   => { let v = field.text().await.unwrap_or_default();
                             timeout   = v.parse().unwrap_or(60); }
            "max_paths" => { let v = field.text().await.unwrap_or_default();
                             max_paths = v.parse().unwrap_or(100); }
            _           => { let _ = field.text().await; }
        }
    }

    if c_bytes.is_empty() || r_bytes.is_empty() || function.is_empty() {
        let body = serde_json::to_string(&Msg::Error {
            text: "Missing required fields: c_file, rust_file, function".into(),
        }).unwrap_or_default();
        return (StatusCode::BAD_REQUEST, [("content-type", "application/x-ndjson")], body);
    }

    let tmpdir = tempfile::tempdir().unwrap();
    let c_path = tmpdir.path().join(&c_name);
    let r_path = tmpdir.path().join(&r_name);
    std::fs::write(&c_path, &c_bytes).unwrap();
    std::fs::write(&r_path, &r_bytes).unwrap();

    let parsed_bounds = match parse_bounds(&bounds) {
        Ok(b)  => b,
        Err(e) => {
            let body = serde_json::to_string(&Msg::Error { text: e.to_string() }).unwrap_or_default();
            return (StatusCode::BAD_REQUEST, [("content-type", "application/x-ndjson")], body);
        }
    };

    let config = AnalysisConfig {
        c_file:        c_path.to_string_lossy().to_string(),
        rust_file:     r_path.to_string_lossy().to_string(),
        function_name: function.clone(),
        bounds:        parsed_bounds,
        max_paths,
        timeout,
    };

    let msgs = tokio::task::spawn_blocking(move || run_pipeline(config))
        .await
        .unwrap_or_else(|e| vec![Msg::Error { text: e.to_string() }]);

    let body: String = msgs
        .iter()
        .filter_map(|m| serde_json::to_string(m).ok())
        .collect::<Vec<_>>()
        .join("\n");

    (StatusCode::OK, [("content-type", "application/x-ndjson")], body)
}

// ── Pipeline ──────────────────────────────────────────

fn run_pipeline(config: AnalysisConfig) -> Vec<Msg> {
    let mut msgs: Vec<Msg> = Vec::new();

    macro_rules! log {
        ($level:expr, $text:expr) => {
            msgs.push(Msg::Log { level: $level.to_string(), text: $text.to_string() });
        };
    }

    log!("step", "[ Step 1/7 ] Input Validation...");
    let validation = match crate::validator::validate(&config) {
        Ok(v)  => v,
        Err(e) => { msgs.push(Msg::Error { text: e.to_string() }); return msgs; }
    };
    if !validation.success {
        for err in &validation.errors { log!("err", format!("  ✗ {}", err)); }
        msgs.push(Msg::Error { text: "Validation failed".into() });
        return msgs;
    }
    log!("ok", "  ✓ Validation passed");

    log!("step", "[ Step 2/7 ] Compiling to LLVM IR...");
    let ir_files = match crate::compiler::compile(&config) {
        Ok(f)  => f,
        Err(e) => { msgs.push(Msg::Error { text: e.to_string() }); return msgs; }
    };
    log!("ok", format!("  ✓ C IR:    {}", ir_files.c_ir_path));
    log!("ok", format!("  ✓ Rust IR: {}", ir_files.rust_ir_path));

    log!("step", "[ Step 3/7 ] Normalizing IR...");
    let normalized = match crate::normalizer::normalize(&config, &ir_files) {
        Ok(n)  => n,
        Err(e) => { msgs.push(Msg::Error { text: e.to_string() }); return msgs; }
    };
    log!("ok", "  ✓ Normalization complete");

    log!("step", "[ Step 4/7 ] Instrumenting IR...");
    let instrumented = match crate::instrumentor::instrument(&config, &normalized) {
        Ok(i)  => i,
        Err(e) => { msgs.push(Msg::Error { text: e.to_string() }); return msgs; }
    };
    log!("ok", "  ✓ Instrumentation complete");

    log!("step", "[ Step 5/7 ] Running Symbolic Execution (KLEE)...");
    let summaries = match crate::symbolic::execute(&config, &instrumented) {
        Ok(s)  => s,
        Err(e) => { msgs.push(Msg::Error { text: e.to_string() }); return msgs; }
    };
    log!("ok", format!("  ✓ C paths:    {}", summaries.c_summaries.len()));
    log!("ok", format!("  ✓ Rust paths: {}", summaries.rust_summaries.len()));

    log!("step", "[ Step 6/7 ] Checking Equivalence...");
    let result = match crate::equivalence::check(
        &config,
        &ir_files,
        &summaries.c_summaries,
        &summaries.rust_summaries,
    ) {
        Ok(r)  => r,
        Err(e) => { msgs.push(Msg::Error { text: e.to_string() }); return msgs; }
    };

    log!("step", "[ Step 7/7 ] Generating Report...");
    match crate::reporter::generate(&config, &result) {
        Ok(path) => { log!("ok",   format!("  ✓ Report: {}", path)); }
        Err(e)   => { log!("warn", format!("  ⚠ Report failed: {}", e)); }
    }

    // ── Build counterexample message ──────────────────
    let ce = result.counterexample.as_ref().map(|c| CeMsg {
        inputs:   c.inputs.clone(),
        c_return: c.c_behavior.return_value.clone(),
        r_return: c.rust_behavior.return_value.clone(),
    });

    // ── Build SEMANTIC diff (from KLEE constraints) ───
    // Only when NOT equivalent.
    // We use crate::diff::find_semantic_divergence which compares
    // KLEE path constraints — NOT source text lines.
    let diff: Option<SemanticDiffMsg> = if result.verdict != Verdict::Equivalent {
        let c_src    = std::fs::read_to_string(&config.c_file).unwrap_or_default();
        let rust_src = std::fs::read_to_string(&config.rust_file).unwrap_or_default();

        let sem = crate::diff::find_semantic_divergence(
            result.c_path.as_ref(),
            result.rust_path.as_ref(),
            &c_src,
            &rust_src,
        );

        let extra_rust = result.statistics.total_paths_rust
            .saturating_sub(result.statistics.total_paths_c);

        Some(build_semantic_diff_msg(sem, &c_src, &rust_src, extra_rust))
    } else {
        None
    };

    msgs.push(Msg::Result {
        equivalent:     result.verdict == Verdict::Equivalent,
        paths_c:        result.statistics.total_paths_c,
        paths_rust:     result.statistics.total_paths_rust,
        inputs_tested:  result.statistics.merged_pairs,
        counterexample: ce,
        diff,
        time_taken:     result.time_taken,
    });

    msgs
}

// ── Semantic diff → wire message ──────────────────────

/// Convert the output of `crate::diff::find_semantic_divergence`
/// into the JSON struct the browser renders.
fn build_semantic_diff_msg(
    sem:      Option<crate::diff::SemanticDiff>,
    c_src:    &str,
    rust_src: &str,
    extra_rust_paths: usize,
) -> SemanticDiffMsg {
    let c_lines_raw:    Vec<&str> = c_src.lines().collect();
    let rust_lines_raw: Vec<&str> = rust_src.lines().collect();

    match sem {
        // ── We have a KLEE-derived semantic diff ──────
        Some(sd) => {
            let div_c    = sd.c_line.as_ref().map(|(n, _)| *n);    // 1-based
            let div_rust = sd.rust_line.as_ref().map(|(n, _)| *n);

            SemanticDiffMsg {
                condition_c:       sd.human_c.clone(),
                condition_rust:    sd.human_rust.clone(),
                diverge_line_c:    div_c,
                diverge_line_rust: div_rust,
                c_lines:    make_window(&c_lines_raw,    div_c.map(|n| n - 1)),
                rust_lines: make_window(&rust_lines_raw, div_rust.map(|n| n - 1)),
                suggestion: sd.suggestion.clone(),
                extra_rust_paths,
            }
        }

        // ── KLEE constraints didn't yield a diff ─────
        // Fall back to showing both full sources side-by-side
        // with no line highlighted. The suggestion explains why.
        None => SemanticDiffMsg {
            condition_c:       "—".into(),
            condition_rust:    "—".into(),
            diverge_line_c:    None,
            diverge_line_rust: None,
            c_lines:    make_window(&c_lines_raw,    None),
            rust_lines: make_window(&rust_lines_raw, None),
            suggestion: "KLEE path constraints were insufficient to locate the exact divergent \
                         condition automatically.\n\
                         Review the counterexample inputs above and compare the source files \
                         side-by-side below.".into(),
            extra_rust_paths,
        },
    }
}

/// Build a window of lines (±6 around highlight, capped at 60 total).
fn make_window(lines: &[&str], highlight: Option<usize>) -> Vec<DiffLine> {
    let max = 60usize;
    let start = if let Some(h) = highlight {
        h.saturating_sub(6).min(lines.len().saturating_sub(max))
    } else { 0 };
    let end = (start + max).min(lines.len());

    (start..end).map(|i| DiffLine {
        num:       i + 1,
        text:      lines[i].to_string(),
        highlight: highlight == Some(i),
    }).collect()
}

// ── Helpers ───────────────────────────────────────────

fn parse_bounds(s: &str) -> Result<Vec<InputBound>> {
    let mut out = Vec::new();
    for part in s.split(',') {
        let p: Vec<&str> = part.trim().split(':').collect();
        if p.len() != 3 {
            return Err(anyhow::anyhow!("Invalid bounds '{}'. Use name:min:max", part));
        }
        out.push(InputBound {
            name: p[0].to_string(),
            min:  p[1].parse::<i64>()?,
            max:  p[2].parse::<i64>()?,
        });
    }
    Ok(out)
}