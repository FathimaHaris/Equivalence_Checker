// src/reporter/mod.rs
// ═══════════════════════════════════════════════════════
// Module 7: Report Generation
// Creates human-readable reports of verification results
// ═══════════════════════════════════════════════════════

use crate::types::{AnalysisConfig, EquivalenceResult, Verdict,TestCaseResult};
use anyhow::Result;
use std::fs;

/// Generate verification report
pub fn generate(config: &AnalysisConfig, result: &EquivalenceResult) -> Result<String> {
    // Create output directory
    fs::create_dir_all("output")?;

    // Generate HTML report
    let html_path = format!("output/{}_report.html", config.function_name);
    let html_content = generate_html_report(config, result);
    fs::write(&html_path, html_content)?;

    // Generate JSON report (machine-readable)
    let json_path = format!("output/{}_report.json", config.function_name);
    let json_content = serde_json::to_string_pretty(result)?;
    fs::write(&json_path, json_content)?;

    Ok(html_path)
}

/// Generate HTML report
fn generate_html_report(config: &AnalysisConfig, result: &EquivalenceResult) -> String {
    let verdict_color = match result.verdict {
        Verdict::Equivalent => "#10b981",
        Verdict::NotEquivalent => "#ef4444",
        Verdict::Unknown => "#f59e0b",
    };

    let verdict_text = match result.verdict {
        Verdict::Equivalent => "✓ SEMANTICALLY EQUIVALENT",
        Verdict::NotEquivalent => "✗ NOT EQUIVALENT",
        Verdict::Unknown => "? UNKNOWN",
    };

    format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Equivalence Check Report - {}</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
            background: #f3f4f6;
            padding: 40px 20px;
        }}
        .container {{
            max-width: 900px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 4px 20px rgba(0,0,0,0.1);
            overflow: hidden;
        }}
        .header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 40px;
            text-align: center;
        }}
        .header h1 {{
            font-size: 28px;
            margin-bottom: 10px;
        }}
        .header p {{
            opacity: 0.9;
            font-size: 14px;
        }}
        .verdict {{
            padding: 40px;
            text-align: center;
            background: {};
            color: white;
        }}
        .verdict h2 {{
            font-size: 32px;
            font-weight: 700;
        }}
        .content {{
            padding: 40px;
        }}
        .info-grid {{
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
            margin-bottom: 30px;
        }}
        .info-box {{
            background: #f9fafb;
            padding: 20px;
            border-radius: 8px;
            border-left: 4px solid #667eea;
        }}
        .info-box h3 {{
            font-size: 12px;
            text-transform: uppercase;
            color: #6b7280;
            margin-bottom: 8px;
        }}
        .info-box p {{
            font-size: 18px;
            font-weight: 600;
            color: #111827;
        }}
        .counterexample {{
            background: #fef2f2;
            border: 1px solid #fecaca;
            border-radius: 8px;
            padding: 20px;
            margin-top: 20px;
        }}
        .counterexample h3 {{
            color: #dc2626;
            margin-bottom: 10px;
        }}
        .footer {{
            padding: 20px 40px;
            background: #f9fafb;
            text-align: center;
            color: #6b7280;
            font-size: 12px;
        }}
        code {{
            background: #f3f4f6;
            padding: 2px 6px;
            border-radius: 4px;
            font-family: "Monaco", "Courier New", monospace;
            font-size: 13px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Equivalence Check Report</h1>
            <p>LLVM-Based Semantic Equivalence Verification</p>
        </div>

        <div class="verdict">
            <h2>{}</h2>
        </div>

        <div class="content">
            <div class="info-grid">
                <div class="info-box">
                    <h3>Function Analyzed</h3>
                    <p><code>{}</code></p>
                </div>
                <div class="info-box">
                    <h3>C Source</h3>
                    <p>{}</p>
                </div>
                <div class="info-box">
                    <h3>Rust Source</h3>
                    <p>{}</p>
                </div>
                <div class="info-box">
                    <h3>Paths Compared</h3>
                    <p>{}</p>
                </div>
            </div>

            <div class="info-box">
                <h3>Analysis Time</h3>
                <p>{:.2} seconds</p>
            </div>

            {}
        </div>

        <div class="footer">
            Generated by Equivalence Checker v0.1.0 | MCA Project - Fathima A A
        </div>
    </div>
</body>
</html>"#,
        config.function_name,
        verdict_color,
        verdict_text,
        config.function_name,
        config.c_file,
        config.rust_file,
        result.paths_compared,
        result.time_taken,
        // generate_counterexample_html(result)
        format!(
        "{}{}{}",
        generate_test_table_html(&result.test_cases),
        generate_observables_html(&result.test_cases),
        generate_counterexample_html(result)
    )

    )
}

/// Generate counterexample section HTML
fn generate_counterexample_html(result: &EquivalenceResult) -> String {
    if let Some(ce) = &result.counterexample {
        let inputs_html: Vec<String> = ce.inputs.iter()
            .map(|(name, val)| format!("<li><code>{} = {}</code></li>", name, val))
            .collect();

        format!(r#"
            <div class="counterexample">
                <h3>⚠ Counterexample Found</h3>
                <p>The programs differ for the following inputs:</p>
                <ul>
                    {}
                </ul>
            </div>
        "#, inputs_html.join("\n"))
    } else {
        String::new()
    }
}



fn generate_test_table_html(test_cases: &[TestCaseResult]) -> String {
    if test_cases.is_empty() {
        return String::new();
    }

    let rows: Vec<String> = test_cases.iter().map(|tc| {
        let x = tc.inputs.iter().find(|(k,_)| k == "x").map(|(_,v)| *v).unwrap_or(0);
        let y = tc.inputs.iter().find(|(k,_)| k == "y").map(|(_,v)| *v).unwrap_or(0);

        let c_ret = &tc.c_behavior.return_value;
        let r_ret = &tc.rust_behavior.return_value;

        let ok = tc.differences.is_empty();
        let status = if ok { "✓" } else { "✗" };

        // highlight failing row
        let row_style = if ok { "" } else { "style=\"background:#fef2f2;\"" };

        format!(
            "<tr {row_style}><td>({x},{y})</td><td><code>{}</code></td><td><code>{}</code></td><td style=\"font-weight:700\">{}</td></tr>",
            html_escape(c_ret),
            html_escape(r_ret),
            status,
            // result.time_taken,
            
        )
    }).collect();

    format!(r#"
    <div style="margin-top:30px;">
      <h3 style="margin-bottom:10px;">Test Case Results</h3>
      <table style="width:100%; border-collapse: collapse; font-size:14px;">
        <thead>
          <tr style="background:#f3f4f6;">
            <th style="text-align:left; padding:10px; border:1px solid #e5e7eb;">Input</th>
            <th style="text-align:left; padding:10px; border:1px solid #e5e7eb;">C Return</th>
            <th style="text-align:left; padding:10px; border:1px solid #e5e7eb;">Rust Return</th>
            <th style="text-align:left; padding:10px; border:1px solid #e5e7eb;">Status</th>
          </tr>
        </thead>
        <tbody>
          {}
        </tbody>
      </table>
    </div>
    "#, rows.join("\n"))
}

/// very small safe escape (good enough for your strings)
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}


fn generate_observables_html(test_cases: &[TestCaseResult]) -> String {
    if test_cases.is_empty() {
        return String::new();
    }

    let blocks: Vec<String> = test_cases.iter().map(|tc| {
        let input_str = tc.inputs.iter()
            .map(|(k,v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(", ");

        let c_stdout = tc.c_behavior.stdout.join("\\n");
        let r_stdout = tc.rust_behavior.stdout.join("\\n");

        let c_globals = tc.c_behavior.globals.iter()
            .map(|(k,v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(", ");

        let r_globals = tc.rust_behavior.globals.iter()
            .map(|(k,v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(", ");

        let diff_text = if tc.differences.is_empty() {
            "EQUIVALENT ✓".to_string()
        } else {
            let d = tc.differences.iter().map(|d| {
                format!("{:?}: C={} vs Rust={}", d.kind, d.c_value, d.rust_value)
            }).collect::<Vec<_>>().join(" | ");
            format!("NOT EQUIVALENT ✗ — {d}")
        };

        format!(r#"
        <div style="margin-top:16px; padding:16px; border:1px solid #e5e7eb; border-radius:8px;">
          <div style="font-weight:700; margin-bottom:8px;">Input: <code>{}</code></div>

          <div style="display:grid; grid-template-columns:1fr 1fr; gap:12px;">
            <div style="background:#f9fafb; padding:12px; border-radius:8px;">
              <div style="font-weight:700; margin-bottom:6px;">C Program</div>
              <div>Return: <code>{}</code></div>
              <div>stdout: <code>{}</code></div>
              <div>globals: <code>{}</code></div>
            </div>

            <div style="background:#f9fafb; padding:12px; border-radius:8px;">
              <div style="font-weight:700; margin-bottom:6px;">Rust Program</div>
              <div>Return: <code>{}</code></div>
              <div>stdout: <code>{}</code></div>
              <div>globals: <code>{}</code></div>
            </div>
          </div>

          <div style="margin-top:10px; font-weight:700;">{}</div>
        </div>
        "#,
        html_escape(&input_str),
        html_escape(&tc.c_behavior.return_value),
        html_escape(&c_stdout),
        html_escape(&c_globals),
        html_escape(&tc.rust_behavior.return_value),
        html_escape(&r_stdout),
        html_escape(&r_globals),
        html_escape(&diff_text),
        )
    }).collect();

    format!(r#"
      <div style="margin-top:30px;">
        <h3 style="margin-bottom:10px;">Step 6.3 — Collected Observables</h3>
        {}
      </div>
    "#, blocks.join("\n"))
}