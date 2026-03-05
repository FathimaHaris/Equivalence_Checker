// src/reporter/mod.rs
use crate::types::{AnalysisConfig, EquivalenceResult, Verdict};
use crate::diff::{find_semantic_divergence, SemanticDiff};
use anyhow::Result;
use std::fs;

pub fn generate(config: &AnalysisConfig, result: &EquivalenceResult) -> Result<String> {
    fs::create_dir_all("output")?;
    let html_path = format!("output/{}_report.html", config.function_name);
    fs::write(&html_path, generate_html_report(config, result))?;
    let json_path = format!("output/{}_report.json", config.function_name);
    fs::write(&json_path, serde_json::to_string_pretty(result)?)?;
    Ok(html_path)
}

fn generate_html_report(config: &AnalysisConfig, result: &EquivalenceResult) -> String {
    let c_src    = fs::read_to_string(&config.c_file).unwrap_or_default();
    let rust_src = fs::read_to_string(&config.rust_file).unwrap_or_default();

    // Find semantic divergence from KLEE constraints (not text diff)
    let sem_diff = find_semantic_divergence(
        result.c_path.as_ref(),
        result.rust_path.as_ref(),
        &c_src,
        &rust_src,
    );

    let diff_html       = build_diff_html(&c_src, &rust_src, &sem_diff);
    // let suggestion_html = build_suggestion_html(&sem_diff, result);
    // let suggestion_html = String::new();
    let ce_html         = generate_counterexample_html(result);
    let stats_html      = generate_stats_html(result);
    let path_note       = generate_path_note_html(result);

    let (vborder, vcolor, vicon, vtext) = match result.verdict {
        Verdict::Equivalent    => ("rgba(5,150,105,.3)",  "#059669", "✓", "SEMANTICALLY EQUIVALENT"),
        Verdict::NotEquivalent => ("rgba(220,38,38,.3)",  "#dc2626", "✗", "NOT EQUIVALENT"),
        Verdict::Unknown       => ("rgba(217,119,6,.3)",  "#d97706", "?", "UNKNOWN"),
    };

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>EQ·CHECK — {fn_name}</title>
<link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;600&family=Inter:wght@400;500;600&display=swap" rel="stylesheet">
<style>
*,*::before,*::after{{box-sizing:border-box;margin:0;padding:0}}
body{{background:#0a0c0f;color:#c8d0dc;font-family:'Inter',sans-serif;font-size:14px;padding:40px 24px 80px}}
.wrap{{max-width:1100px;margin:0 auto}}
.header{{text-align:center;margin-bottom:36px}}
.header-title{{font-family:'JetBrains Mono',monospace;font-size:26px;font-weight:600;letter-spacing:.08em;color:#fff}}
.header-title span{{color:#00e5ff}}
.header-sub{{color:#4a5568;font-size:13px;margin-top:6px}}
.verdict{{border-radius:12px;padding:28px;text-align:center;margin-bottom:28px;border:1px solid {vborder};background:rgba(255,255,255,.02)}}
.verdict-icon{{font-size:44px}}
.verdict-text{{font-family:'JetBrains Mono',monospace;font-size:22px;font-weight:700;color:{vcolor};margin-top:8px}}
.verdict-sub{{color:#4a5568;font-size:13px;margin-top:5px}}
.stats-row{{display:flex;gap:14px;flex-wrap:wrap;margin-bottom:24px}}
.stat-box{{flex:1;min-width:110px;background:#111318;border:1px solid #2a3040;border-radius:10px;padding:16px;text-align:center}}
.stat-val{{font-family:'JetBrains Mono',monospace;font-size:22px;font-weight:700;color:#fff}}
.stat-key{{font-size:11px;color:#4a5568;text-transform:uppercase;letter-spacing:.06em;margin-top:4px}}
.section{{background:#111318;border:1px solid #2a3040;border-radius:12px;padding:22px;margin-bottom:22px}}
.section-title{{font-family:'JetBrains Mono',monospace;font-size:11px;font-weight:600;color:#00e5ff;letter-spacing:.1em;text-transform:uppercase;margin-bottom:14px}}
.path-note{{background:rgba(255,209,102,.05);border:1px solid rgba(255,209,102,.2);border-radius:8px;padding:12px 16px;font-size:13px;color:#ffd166;margin-bottom:22px}}
.ce-grid{{display:grid;grid-template-columns:1fr 1fr;gap:14px}}
.ce-box{{background:rgba(255,255,255,.03);border:1px solid #2a3040;border-radius:8px;padding:14px}}
.ce-box-title{{font-family:'JetBrains Mono',monospace;font-size:11px;color:#4a5568;text-transform:uppercase;letter-spacing:.08em;margin-bottom:8px}}
.ce-val{{font-family:'JetBrains Mono',monospace;font-size:18px;font-weight:700}}
.c-val{{color:#00e096}} .r-val{{color:#ff4d6d}} .in-val{{color:#ffd166}}
.diff-grid{{display:grid;grid-template-columns:1fr 1fr;gap:2px;border-radius:10px;overflow:hidden;border:1px solid #1e2229}}
.diff-panel{{background:#060809;font-family:'JetBrains Mono',monospace;font-size:12px}}
.diff-hdr{{background:#0d1015;padding:9px 14px;font-size:11px;color:#4a5568;border-bottom:1px solid #1e2229;display:flex;align-items:center;gap:8px}}
.badge{{padding:2px 8px;border-radius:4px;font-size:10px;font-weight:700}}
.badge-c{{background:rgba(0,229,255,.1);color:#00e5ff;border:1px solid rgba(0,229,255,.2)}}
.badge-r{{background:rgba(255,77,109,.1);color:#ff8c9e;border:1px solid rgba(255,77,109,.25)}}
.diff-body{{overflow-x:auto;max-height:500px;overflow-y:auto}}
table{{border-collapse:collapse;width:100%}}
.hl{{background:rgba(255,77,109,.12);border-left:3px solid #ff4d6d}}
.hl .dc{{color:#fff}}
.ln{{width:40px;padding:3px 8px;text-align:right;color:#2a3040;user-select:none;font-size:11px}}
.dc{{padding:3px 12px;white-space:pre;color:#4a5568;font-size:12px;line-height:1.6}}
.ar{{width:22px;text-align:center;color:#ff4d6d;font-size:12px}}
.diff-label{{background:rgba(255,77,109,.15);color:#ff4d6d;font-size:9px;padding:1px 5px;border-radius:3px;margin-left:6px;font-family:'JetBrains Mono',monospace;vertical-align:middle}}
.smt-row{{display:flex;gap:10px;align-items:flex-start;margin-bottom:8px;font-family:'JetBrains Mono',monospace;font-size:13px}}
.smt-label{{color:#4a5568;font-size:11px;width:80px;flex-shrink:0;padding-top:2px}}
.smt-val{{color:#c8d0dc}}
.smt-val.diverge{{color:#ffd166;font-weight:600}}
pre.suggestion{{background:#060809;border:1px solid rgba(0,229,255,.12);border-radius:8px;padding:16px;font-family:'JetBrains Mono',monospace;font-size:12px;color:#c8d0dc;line-height:1.8;white-space:pre-wrap;word-break:break-word}}
.footer{{text-align:center;color:#2a3040;font-size:12px;margin-top:36px}}
</style>
</head>
<body>
<div class="wrap">
  <div class="header">
    <div class="header-title">EQ<span>·</span>CHECK Report</div>
    <div class="header-sub">C ↔ Rust Semantic Equivalence · <code style="color:#ffd166">{fn_name}()</code></div>
  </div>

  <div class="verdict">
    <div class="verdict-icon">{vicon}</div>
    <div class="verdict-text">{vtext}</div>
    <div class="verdict-sub">{c_file} &nbsp;↔&nbsp; {rust_file}</div>
  </div>

  {stats_html}
  {path_note}
  {ce_html}
  {diff_html}


  <div class="footer">EQ·CHECK · C is the source of truth · Rust is the migration under verification</div>
</div>
</body>
</html>"#,
        fn_name      = config.function_name,
        c_file       = html_escape(&config.c_file),
        rust_file    = html_escape(&config.rust_file),
        vborder      = vborder,
        vcolor       = vcolor,
        vicon        = vicon,
        vtext        = vtext,
        stats_html   = stats_html,
        path_note    = path_note,
        ce_html      = ce_html,
        diff_html    = diff_html,
        // suggestion_html = suggestion_html,
    )
}

fn generate_stats_html(result: &EquivalenceResult) -> String {
    format!(r#"<div class="stats-row">
      <div class="stat-box"><div class="stat-val">{}</div><div class="stat-key">C Paths</div></div>
      <div class="stat-box"><div class="stat-val">{}</div><div class="stat-key">Rust Paths</div></div>
      <div class="stat-box"><div class="stat-val">{}</div><div class="stat-key">Inputs Tested</div></div>
      <div class="stat-box"><div class="stat-val">{:.2}s</div><div class="stat-key">Time</div></div>
    </div>"#,
        result.statistics.total_paths_c,
        result.statistics.total_paths_rust,
        result.statistics.merged_pairs,
        result.time_taken,
    )
}

fn generate_path_note_html(result: &EquivalenceResult) -> String {
    let extra = result.statistics.total_paths_rust
        .saturating_sub(result.statistics.total_paths_c);
    if extra == 0 { return String::new(); }
    format!(r#"<div class="path-note">⚠ Rust has {} more path(s) than C. This is expected — Rust's compiler inserts implicit safety checks (integer overflow guards, bounds checks) that create extra branches absent in C. These are <strong>not</strong> bugs.</div>"#, extra)
}

fn generate_counterexample_html(result: &EquivalenceResult) -> String {
    let ce = match &result.counterexample { Some(c) => c, None => return String::new() };
    let inputs = ce.inputs.iter().map(|(k, v)|
        format!(r#"<div style="margin-bottom:5px"><span style="color:#4a5568;font-family:'JetBrains Mono',monospace;font-size:12px;display:inline-block;width:50px">{}</span><span class="ce-val in-val"> = {}</span></div>"#, k, v)
    ).collect::<String>();

    format!(r#"<div class="section">
      <div class="section-title">⚡ Counterexample</div>
      <div class="ce-grid">
        <div class="ce-box"><div class="ce-box-title">Diverging Input</div>{}</div>
        <div class="ce-box" style="display:grid;grid-template-columns:1fr 1fr;gap:12px;align-items:center">
          <div><div class="ce-box-title">C Returns</div><div class="ce-val c-val">{}</div></div>
          <div><div class="ce-box-title">Rust Returns</div><div class="ce-val r-val">{}</div></div>
        </div>
      </div>
    </div>"#,
        inputs,
        html_escape(&ce.c_behavior.return_value),
        html_escape(&ce.rust_behavior.return_value),
    )
}

fn build_diff_html(
    c_src:    &str,
    rust_src: &str,
    sem_diff: &Option<crate::diff::SemanticDiff>,
) -> String {
    let c_lines:    Vec<&str> = c_src.lines().collect();
    let rust_lines: Vec<&str> = rust_src.lines().collect();

    // Highlight the semantically divergent line if found
    let hl_c    = sem_diff.as_ref().and_then(|d| d.c_line.as_ref().map(|(n,_)| n - 1));
    let hl_rust = sem_diff.as_ref().and_then(|d| d.rust_line.as_ref().map(|(n,_)| n - 1));

    // Also show the SMT constraint comparison if available
    let constraint_html = if let Some(sd) = sem_diff {
        format!(r#"<div style="margin-bottom:14px;background:rgba(255,255,255,.02);border:1px solid #2a3040;border-radius:8px;padding:14px">
          <div style="font-family:'JetBrains Mono',monospace;font-size:11px;color:#4a5568;text-transform:uppercase;letter-spacing:.08em;margin-bottom:10px">Divergent Branch Condition</div>
          <div class="smt-row"><span class="smt-label">C:</span><span class="smt-val diverge">{}</span></div>
          <div class="smt-row"><span class="smt-label">Rust:</span><span class="smt-val diverge">{}</span></div>
          <div style="font-size:11px;color:#4a5568;margin-top:8px">These conditions produce different results when the counterexample inputs are substituted.</div>
        </div>"#,
            html_escape(&sd.human_c),
            html_escape(&sd.human_rust),
        )
    } else { String::new() };

    let c_rows    = render_rows(&c_lines,    hl_c);
    let rust_rows = render_rows(&rust_lines, hl_rust);

    format!(r#"<div class="section">
      <div class="section-title">⟨/⟩ Source Comparison</div>
      {constraint_html}
      <div class="diff-grid">
        <div class="diff-panel">
          <div class="diff-hdr"><span class="badge badge-c">C</span> Source of truth</div>
          <div class="diff-body"><table>{c_rows}</table></div>
        </div>
        <div class="diff-panel">
          <div class="diff-hdr"><span class="badge badge-r">RUST</span> Migration under verification</div>
          <div class="diff-body"><table>{rust_rows}</table></div>
        </div>
      </div>
    </div>"#,
        constraint_html = constraint_html,
        c_rows          = c_rows,
        rust_rows       = rust_rows,
    )
}

fn render_rows(lines: &[&str], highlight: Option<usize>) -> String {
    let max   = 60usize;
    let start = if let Some(h) = highlight {
        h.saturating_sub(4).min(lines.len().saturating_sub(max))
    } else { 0 };
    let end = (start + max).min(lines.len());

    (start..end).map(|i| {
        let hl    = highlight == Some(i);
        let label = if hl { r#"<span class="diff-label">DIFFERS</span>"# } else { "" };
        let arrow = if hl { r#"<td class="ar">←</td>"# } else { r#"<td class="ar"></td>"# };
        format!(
            r#"<tr class="{}"><td class="ln">{}</td><td class="dc">{}{}</td>{}</tr>"#,
            if hl { "hl" } else { "" },
            i + 1,
            html_escape(lines[i]),
            label,
            arrow,
        )
    }).collect()
}

fn build_suggestion_html(
    sem_diff: &Option<crate::diff::SemanticDiff>,
    result:   &EquivalenceResult,
) -> String {
    if result.verdict == Verdict::Equivalent { return String::new(); }

    let suggestion = if let Some(sd) = sem_diff {
        sd.suggestion.clone()
    } else {
        "No specific semantic divergence detected from KLEE constraints.\nReview the counterexample inputs above and compare logic manually.".into()
    };

    format!(r#"<div class="section">
      <div class="section-title">✏ Suggested Fix (C is source of truth)</div>
      <pre class="suggestion">{}</pre>
    </div>"#, html_escape(&suggestion))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}