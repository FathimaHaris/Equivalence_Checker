// src/server/mod.rs
// ═══════════════════════════════════════════════════════
// Tiny Axum web server that:
//   GET  /           → serves static/index.html
//   POST /run        → accepts multipart form, runs the full
//                      equivalence-checking pipeline, streams
//                      stdout back as newline-delimited JSON
// ═══════════════════════════════════════════════════════

use axum::{
    extract::Multipart,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};
// use tokio::sync::mpsc;
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

    // Try to open browser automatically
    let url = format!("http://localhost:{}", port);
    let _ = std::process::Command::new("xdg-open").arg(&url).spawn();   // Linux
    let _ = std::process::Command::new("open").arg(&url).spawn();       // macOS

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// ── GET / ─────────────────────────────────────────────

async fn serve_ui() -> impl IntoResponse {
    // Read the HTML file at runtime so you can edit it without recompiling
    let html = std::fs::read_to_string("static/index.html")
        .unwrap_or_else(|_| {
            "<h1>UI not found</h1><p>Place <code>static/index.html</code> in your project root.</p>"
                .to_string()
        });

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"));
    (headers, html)
}

// ── POST /run ─────────────────────────────────────────

/// Every message sent back to the browser is one of these.
#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Msg {
    /// A line of terminal output
    Log { level: String, text: String },
    /// Final structured result
    Result {
        equivalent: bool,
        paths_c:    usize,
        paths_rust: usize,
        inputs_tested: usize,
        counterexample: Option<CeMsg>,
        time_taken: f64,
    },
    /// Something went wrong
    Error { text: String },
}

#[derive(Serialize, Clone)]
pub struct CeMsg {
    pub inputs:   Vec<(String, i64)>,
    pub c_return: String,
    pub r_return: String,
}

async fn run_check(mut multipart: Multipart) -> impl IntoResponse {
    // ── Parse multipart form ──────────────────────────
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
            "c_file" => {
                c_name  = field.file_name().unwrap_or("program.c").to_string();
                c_bytes = field.bytes().await.unwrap_or_default().to_vec();
            }
            "rust_file" => {
                r_name  = field.file_name().unwrap_or("program.rs").to_string();
                r_bytes = field.bytes().await.unwrap_or_default().to_vec();
            }
            "function"  => { function  = field.text().await.unwrap_or_default(); }
            "bounds"    => { bounds    = field.text().await.unwrap_or(bounds); }
            "timeout"   => {
                let v = field.text().await.unwrap_or_default();
                timeout = v.parse().unwrap_or(60);
            }
            "max_paths" => {
                let v = field.text().await.unwrap_or_default();
                max_paths = v.parse().unwrap_or(100);
            }
            _ => { let _ = field.text().await; }
        }
    }

    // Validate required fields
    if c_bytes.is_empty() || r_bytes.is_empty() || function.is_empty() {
        let body = serde_json::to_string(&Msg::Error {
            text: "Missing required fields: c_file, rust_file, function".into(),
        })
        .unwrap_or_default();
        return (StatusCode::BAD_REQUEST, [("content-type", "application/x-ndjson")], body);
    }

    // ── Write uploaded files to temp directory ────────
    let tmpdir = tempfile::tempdir().unwrap();
    let c_path = tmpdir.path().join(&c_name);
    let r_path = tmpdir.path().join(&r_name);

    std::fs::write(&c_path, &c_bytes).unwrap();
    std::fs::write(&r_path, &r_bytes).unwrap();

    // ── Parse bounds ──────────────────────────────────
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

    // ── Run the pipeline in a blocking thread ─────────
    // We collect all messages then return them as NDJSON.
    // (For true streaming, replace with SSE — see comment below.)
    let msgs = tokio::task::spawn_blocking(move || {
        run_pipeline(config)
    })
    .await
    .unwrap_or_else(|e| vec![Msg::Error { text: e.to_string() }]);

    // Serialize as newline-delimited JSON (NDJSON)
    let body: String = msgs
        .iter()
        .filter_map(|m| serde_json::to_string(m).ok())
        .collect::<Vec<_>>()
        .join("\n");

    (StatusCode::OK, [("content-type", "application/x-ndjson")], body)
}

// ── Pipeline runner ───────────────────────────────────

fn run_pipeline(config: AnalysisConfig) -> Vec<Msg> {
    let mut msgs: Vec<Msg> = Vec::new();

    macro_rules! log {
        ($level:expr, $text:expr) => {
            msgs.push(Msg::Log { level: $level.to_string(), text: $text.to_string() });
        };
    }

    // Step 1: Validate
    log!("step",  "[ Step 1/7 ] Input Validation...");
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

    // Step 2: Compile
    log!("step", "[ Step 2/7 ] Compiling to LLVM IR...");
    let ir_files = match crate::compiler::compile(&config) {
        Ok(f)  => f,
        Err(e) => { msgs.push(Msg::Error { text: e.to_string() }); return msgs; }
    };
    log!("ok",   format!("  ✓ C IR:    {}", ir_files.c_ir_path));
    log!("ok",   format!("  ✓ Rust IR: {}", ir_files.rust_ir_path));

    // Step 3: Normalize
    log!("step", "[ Step 3/7 ] Normalizing IR...");
    let normalized = match crate::normalizer::normalize(&config, &ir_files) {
        Ok(n)  => n,
        Err(e) => { msgs.push(Msg::Error { text: e.to_string() }); return msgs; }
    };
    log!("ok", "  ✓ Normalization complete");

    // Step 4: Instrument
    log!("step", "[ Step 4/7 ] Instrumenting IR...");
    let instrumented = match crate::instrumentor::instrument(&config, &normalized) {
        Ok(i)  => i,
        Err(e) => { msgs.push(Msg::Error { text: e.to_string() }); return msgs; }
    };
    log!("ok", "  ✓ Instrumentation complete");

    // Step 5: Symbolic execution
    log!("step", "[ Step 5/7 ] Running Symbolic Execution (KLEE)...");
    let summaries = match crate::symbolic::execute(&config, &instrumented) {
        Ok(s)  => s,
        Err(e) => { msgs.push(Msg::Error { text: e.to_string() }); return msgs; }
    };
    log!("ok", format!("  ✓ C paths:    {}", summaries.c_summaries.len()));
    log!("ok", format!("  ✓ Rust paths: {}", summaries.rust_summaries.len()));

    // Step 6: Equivalence
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

    // Step 7: Report
    log!("step", "[ Step 7/7 ] Generating Report...");
    match crate::reporter::generate(&config, &result) {
        Ok(path) => { log!("ok",   format!("  ✓ Report: {}", path)); }
        Err(e)   => { log!("warn", format!("  ⚠ Report failed: {}", e)); }
    }

    // Emit final structured result
    let ce = result.counterexample.as_ref().map(|c| CeMsg {
        inputs:   c.inputs.clone(),
        c_return: c.c_behavior.return_value.clone(),
        r_return: c.rust_behavior.return_value.clone(),
    });

    let inputs_tested = result.statistics.merged_pairs;

    msgs.push(Msg::Result {
        equivalent:    result.verdict == Verdict::Equivalent,
        paths_c:       result.statistics.total_paths_c,
        paths_rust:    result.statistics.total_paths_rust,
        inputs_tested,
        counterexample: ce,
        time_taken:    result.time_taken,
    });

    msgs
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

// ── NOTE: True streaming with SSE ─────────────────────
// If you want log lines to appear in real-time (not after the
// full run), replace the spawn_blocking + NDJSON approach with
// Server-Sent Events using axum's Sse<> response type and a
// tokio::sync::mpsc channel piped through an async task.
// The frontend would then use:
//   const es = new EventSource('/run-stream?...');
//   es.onmessage = e => handleMsg(JSON.parse(e.data));