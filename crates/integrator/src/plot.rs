//! SVG and ASCII plots of f(x) and its antiderivative F(x) or derivative f'(x) over [a, b].

use std::fmt::Write as FmtWrite;
use std::io::Write;

const W: f64 = 600.0;
const H: f64 = 400.0;
const ML: f64 = 60.0;  // left margin
const MR: f64 = 20.0;  // right margin
const MT: f64 = 45.0;  // top margin
const MB: f64 = 50.0;  // bottom margin
const N:  usize = 400; // sample count

/// Controls whether the plot is for integration or differentiation.
/// Affects title text, legend labels, and whether area shading is shown.
#[derive(Clone, Copy, PartialEq)]
pub enum PlotKind {
    Integrate,
    Differentiate,
}

/// Write an SVG plot of `f` (and optionally `secondary`) over [`a`, `b`] to `path`.
#[allow(clippy::too_many_arguments)]
pub fn write_svg(
    path: &str,
    expr_str: &str,
    var: &str,
    a: f64,
    b: f64,
    f: &dyn Fn(f64) -> f64,
    secondary: Option<&dyn Fn(f64) -> f64>,
    numerical: f64,
) -> std::io::Result<()> {
    let svg = render_svg(expr_str, var, a, b, f, secondary, numerical);
    let mut file = std::fs::File::create(path)?;
    file.write_all(svg.as_bytes())
}

/// Render an SVG plot of `f` (and optionally `secondary`) over [`a`, `b`] and return it as a String.
pub fn render_svg(
    expr_str: &str,
    var: &str,
    a: f64,
    b: f64,
    f: &dyn Fn(f64) -> f64,
    secondary: Option<&dyn Fn(f64) -> f64>,
    numerical: f64,
) -> String {
    render_svg_inner(expr_str, var, a, b, f, secondary, numerical, PlotKind::Integrate)
}

/// Render a differentiation SVG plot (f and f') and return it as a String.
pub fn render_svg_diff(
    expr_str: &str,
    var: &str,
    a: f64,
    b: f64,
    f: &dyn Fn(f64) -> f64,
    deriv: Option<&dyn Fn(f64) -> f64>,
) -> String {
    render_svg_inner(expr_str, var, a, b, f, deriv, 0.0, PlotKind::Differentiate)
}

/// Print an ASCII plot of `f` (and optionally `secondary`) over [`a`, `b`] to stdout.
pub fn print_ascii(
    expr_str: &str,
    var: &str,
    a: f64,
    b: f64,
    f: &dyn Fn(f64) -> f64,
    secondary: Option<&dyn Fn(f64) -> f64>,
) {
    println!("{}", render_ascii_string(expr_str, var, a, b, f, secondary));
}

/// Render an ASCII plot of `f` (and optionally `secondary`) over [`a`, `b`] and return it as a String.
pub fn render_ascii_string(
    expr_str: &str,
    var: &str,
    a: f64,
    b: f64,
    f: &dyn Fn(f64) -> f64,
    secondary: Option<&dyn Fn(f64) -> f64>,
) -> String {
    render_ascii(expr_str, var, a, b, f, secondary, PlotKind::Integrate)
}

/// Render a differentiation ASCII plot and return it as a String.
pub fn render_ascii_string_diff(
    expr_str: &str,
    var: &str,
    a: f64,
    b: f64,
    f: &dyn Fn(f64) -> f64,
    deriv: Option<&dyn Fn(f64) -> f64>,
) -> String {
    render_ascii(expr_str, var, a, b, f, deriv, PlotKind::Differentiate)
}

// ── ASCII rendering ───────────────────────────────────────────

const COLS: usize = 60;
const ROWS: usize = 20;

fn render_ascii(
    expr_str: &str,
    var: &str,
    a: f64,
    b: f64,
    f: &dyn Fn(f64) -> f64,
    secondary: Option<&dyn Fn(f64) -> f64>,
    kind: PlotKind,
) -> String {
    // Sample both curves at COLS points
    let xs: Vec<f64> = (0..COLS).map(|i| a + (b - a) * i as f64 / (COLS - 1) as f64).collect();
    let f_ys: Vec<f64>    = xs.iter().map(|&x| f(x)).collect();
    let sec_ys: Option<Vec<f64>> = secondary.map(|g| xs.iter().map(|&x| g(x)).collect());

    // Determine y range
    let mut all_ys: Vec<f64> = f_ys.iter().filter(|y| y.is_finite()).cloned().collect();
    if let Some(ref ay) = sec_ys {
        all_ys.extend(ay.iter().filter(|y| y.is_finite()).cloned());
    }
    if all_ys.is_empty() {
        return "  (no finite values to plot)\n".to_string();
    }
    all_ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let y_lo = all_ys[0];
    let y_hi = all_ys[all_ys.len() - 1];
    let span = (y_hi - y_lo).max(1e-9);
    let pad  = span * 0.05;
    let y_min = y_lo - pad;
    let y_max = y_hi + pad;

    // Map a y value to a row index (0 = top)
    let to_row = |y: f64| -> Option<usize> {
        if !y.is_finite() { return None; }
        let r = ((y_max - y) / (y_max - y_min) * (ROWS - 1) as f64).round() as isize;
        if r < 0 || r >= ROWS as isize { None } else { Some(r as usize) }
    };

    // Build grid
    let mut grid = vec![vec![' '; COLS]; ROWS];

    // Zero line
    if y_min < 0.0 && y_max > 0.0 {
        if let Some(zr) = to_row(0.0) {
            for c in 0..COLS { grid[zr][c] = '-'; }
        }
    }

    // Plot secondary curve first (lower priority)
    if let Some(ref ay) = sec_ys {
        for (c, &y) in ay.iter().enumerate() {
            if let Some(r) = to_row(y) {
                grid[r][c] = 'o';
            }
        }
    }

    // Plot f (higher priority; overlap → '@')
    for (c, &y) in f_ys.iter().enumerate() {
        if let Some(r) = to_row(y) {
            grid[r][c] = if grid[r][c] == 'o' { '@' } else { '*' };
        }
    }

    // Y-axis label width
    let label_w = 8usize;
    let fmt_y = |v: f64| -> String {
        let s = if v.abs() >= 1000.0 || (v.abs() < 0.01 && v != 0.0) {
            format!("{:.1e}", v)
        } else {
            format!("{:.3}", v)
        };
        format!("{:>width$}", s, width = label_w)
    };

    let mut out = String::new();

    // Title and legend
    let (title_prefix, sec_label) = match kind {
        PlotKind::Integrate    => ("∫", format!("F({var})=o")),
        PlotKind::Differentiate => ("d/d{var}", format!("f'({var})=o")),
    };
    let title_prefix = if kind == PlotKind::Differentiate {
        format!("d/d{var}")
    } else {
        title_prefix.to_string()
    };
    let legend = if sec_ys.is_some() {
        format!("f({var})=* | {sec_label} | overlap=@")
    } else {
        format!("f({var})=*")
    };
    writeln!(out, "  {title_prefix}({expr_str})    [{legend}]").unwrap();
    writeln!(out, "  {} ┐", fmt_y(y_max)).unwrap();

    for (r, row) in grid.iter().enumerate() {
        let mid = ROWS / 2;
        let label = if r == 0 {
            fmt_y(y_max)
        } else if r == ROWS - 1 {
            fmt_y(y_min)
        } else if r == mid {
            fmt_y(y_min + (y_max - y_min) * 0.5)
        } else {
            " ".repeat(label_w)
        };

        let row_str: String = row.iter().collect();
        writeln!(out, "  {} │{}", label, row_str).unwrap();
    }

    // X-axis
    let axis_line = "─".repeat(COLS);
    writeln!(out, "  {} └{}", " ".repeat(label_w), axis_line).unwrap();

    // X-axis labels: left, middle, right
    let x_mid = a + (b - a) * 0.5;
    let fmt_x = |v: f64| -> String {
        if v.abs() >= 1000.0 || (v.abs() < 0.01 && v != 0.0) {
            format!("{:.2e}", v)
        } else {
            format!("{:.3}", v)
        }
    };
    let lbl_left  = fmt_x(a);
    let lbl_mid   = fmt_x(x_mid);
    let lbl_right = fmt_x(b);
    let mid_pos   = COLS / 2 - lbl_mid.len() / 2;
    let right_pos = COLS.saturating_sub(lbl_right.len());
    let mut x_row = vec![' '; COLS];
    for (i, c) in lbl_left.chars().enumerate() { if i < COLS { x_row[i] = c; } }
    for (i, c) in lbl_mid.chars().enumerate()  { let p = mid_pos + i;   if p < COLS { x_row[p] = c; } }
    for (i, c) in lbl_right.chars().enumerate(){ let p = right_pos + i; if p < COLS { x_row[p] = c; } }
    let x_row_str: String = x_row.iter().collect();
    writeln!(out, "  {}  {}", " ".repeat(label_w), x_row_str).unwrap();

    out
}

// ── coordinate mapping ────────────────────────────────────────

fn px(x: f64, x_min: f64, x_max: f64) -> f64 {
    ML + (x - x_min) / (x_max - x_min) * (W - ML - MR)
}

fn py(y: f64, y_min: f64, y_max: f64) -> f64 {
    MT + (1.0 - (y - y_min) / (y_max - y_min)) * (H - MT - MB)
}

// ── sampling helpers ─────────────────────────────────────────

fn sample(f: &dyn Fn(f64) -> f64, a: f64, b: f64) -> Vec<(f64, f64)> {
    (0..=N)
        .map(|i| {
            let x = a + (b - a) * i as f64 / N as f64;
            (x, f(x))
        })
        .collect()
}

fn finite_ys<'a>(pts: &'a [(f64, f64)]) -> impl Iterator<Item = f64> + 'a {
    pts.iter().map(|&(_, y)| y).filter(|y| y.is_finite())
}

/// Compute y range with 5% padding.
fn y_range(f_pts: &[(f64, f64)], anti_pts: Option<&[(f64, f64)]>) -> (f64, f64) {
    let mut ys: Vec<f64> = finite_ys(f_pts).collect();
    if let Some(ap) = anti_pts {
        ys.extend(finite_ys(ap));
    }
    if ys.is_empty() {
        return (-1.0, 1.0);
    }
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let lo = ys[0];
    let hi = ys[ys.len() - 1];
    let span = (hi - lo).max(1e-9);
    let pad = span * 0.05;
    (lo - pad, hi + pad)
}

// ── polyline builder (breaks on non-finite values) ───────────

fn polylines(
    pts: &[(f64, f64)],
    x_min: f64, x_max: f64,
    y_min: f64, y_max: f64,
) -> Vec<String> {
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut count = 0usize;

    for &(x, y) in pts {
        if !y.is_finite() || y < y_min || y > y_max {
            if count >= 2 {
                segments.push(current.clone());
            }
            current.clear();
            count = 0;
            continue;
        }
        let sx = px(x, x_min, x_max);
        let sy = py(y, y_min, y_max);
        if !current.is_empty() { current.push(' '); }
        write!(current, "{:.2},{:.2}", sx, sy).unwrap();
        count += 1;
    }
    if count >= 2 {
        segments.push(current);
    }
    segments
}

// ── axis tick helpers ─────────────────────────────────────────

fn nice_ticks(lo: f64, hi: f64, n: usize) -> Vec<f64> {
    let span = hi - lo;
    if span == 0.0 { return vec![lo]; }
    let raw_step = span / (n - 1) as f64;
    let mag = raw_step.log10().floor();
    let step = {
        let s = raw_step / 10f64.powf(mag);
        let s = if s <= 1.0 { 1.0 } else if s <= 2.0 { 2.0 } else if s <= 5.0 { 5.0 } else { 10.0 };
        s * 10f64.powf(mag)
    };
    let first = (lo / step).ceil() * step;
    let mut ticks = Vec::new();
    let mut t = first;
    while t <= hi + step * 1e-9 {
        if t >= lo - step * 1e-9 { ticks.push(t); }
        t += step;
        if ticks.len() > n + 2 { break; }
    }
    ticks
}

fn fmt_tick(v: f64) -> String {
    if v == 0.0 { return "0".to_string(); }
    let abs = v.abs();
    if abs >= 1000.0 || (abs < 0.01 && abs > 0.0) {
        format!("{:.2e}", v)
    } else if (v - v.round()).abs() < 1e-9 * abs.max(1.0) {
        format!("{}", v as i64)
    } else {
        format!("{:.2}", v)
    }
}

// ── main SVG render function ──────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn render_svg_inner(
    expr_str: &str,
    var: &str,
    a: f64,
    b: f64,
    f: &dyn Fn(f64) -> f64,
    secondary: Option<&dyn Fn(f64) -> f64>,
    numerical: f64,
    kind: PlotKind,
) -> String {
    let f_pts   = sample(f, a, b);
    let sec_pts = secondary.map(|g| sample(g, a, b));

    let (y_min, y_max) = y_range(&f_pts, sec_pts.as_deref());
    let x_min = a;
    let x_max = b;

    let mut s = String::new();

    // ── SVG header ────────────────────────────────────────────
    writeln!(s, r##"<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">"##).unwrap();
    writeln!(s, r##"<rect width="{W}" height="{H}" fill="white"/>"##).unwrap();

    // ── clip path ─────────────────────────────────────────────
    let cx = ML; let cy = MT;
    let cw = W - ML - MR; let ch = H - MT - MB;
    writeln!(s, r##"<clipPath id="plot"><rect x="{cx}" y="{cy}" width="{cw}" height="{ch}"/></clipPath>"##).unwrap();

    // ── grid lines ────────────────────────────────────────────
    writeln!(s, r##"<g stroke="#e0e0e0" stroke-width="1">"##).unwrap();
    for t in nice_ticks(x_min, x_max, 6) {
        let sx = px(t, x_min, x_max);
        writeln!(s, r##"<line x1="{sx:.2}" y1="{MT}" x2="{sx:.2}" y2="{:.2}"/>"##, H - MB).unwrap();
    }
    for t in nice_ticks(y_min, y_max, 6) {
        let sy = py(t, y_min, y_max);
        writeln!(s, r##"<line x1="{ML}" y1="{sy:.2}" x2="{:.2}" y2="{sy:.2}"/>"##, W - MR).unwrap();
    }
    writeln!(s, "</g>").unwrap();

    // ── shaded area under f (integration only) ───────────────
    if kind == PlotKind::Integrate {
        let zero_y = py(0.0f64.max(y_min).min(y_max), y_min, y_max);
        let mut pts_str = String::new();
        write!(pts_str, "{:.2},{:.2}", px(a, x_min, x_max), zero_y).unwrap();
        for &(x, y) in &f_pts {
            if !y.is_finite() { continue; }
            let sy = py(y.max(y_min).min(y_max), y_min, y_max);
            write!(pts_str, " {:.2},{:.2}", px(x, x_min, x_max), sy).unwrap();
        }
        write!(pts_str, " {:.2},{:.2}", px(b, x_min, x_max), zero_y).unwrap();
        writeln!(s, r##"<polygon points="{pts_str}" fill="#4a90d9" fill-opacity="0.15" clip-path="url(#plot)"/>"##).unwrap();
    }

    // ── zero axis line ────────────────────────────────────────
    if y_min < 0.0 && y_max > 0.0 {
        let sy = py(0.0, y_min, y_max);
        writeln!(s, r##"<line x1="{ML}" y1="{sy:.2}" x2="{:.2}" y2="{sy:.2}" stroke="#999" stroke-width="1" stroke-dasharray="4,3"/>"##, W - MR).unwrap();
    }

    // ── secondary curve (antiderivative or derivative) ────────
    if let Some(ref ap) = sec_pts {
        for seg in polylines(ap, x_min, x_max, y_min, y_max) {
            writeln!(s, r##"<polyline points="{seg}" fill="none" stroke="#e07020" stroke-width="2" stroke-dasharray="6,4" clip-path="url(#plot)"/>"##).unwrap();
        }
    }

    // ── f curve ───────────────────────────────────────────────
    for seg in polylines(&f_pts, x_min, x_max, y_min, y_max) {
        writeln!(s, r##"<polyline points="{seg}" fill="none" stroke="#1a6bbf" stroke-width="2.5" clip-path="url(#plot)"/>"##).unwrap();
    }

    // ── axes ──────────────────────────────────────────────────
    writeln!(s, r##"<rect x="{ML}" y="{MT}" width="{cw}" height="{ch}" fill="none" stroke="#333" stroke-width="1.5"/>"##).unwrap();

    // ── x-axis ticks & labels ─────────────────────────────────
    writeln!(s, r##"<g font-size="11" font-family="monospace" fill="#333" text-anchor="middle">"##).unwrap();
    for t in nice_ticks(x_min, x_max, 6) {
        let sx = px(t, x_min, x_max);
        let ty = H - MB + 14.0;
        writeln!(s, r##"<line x1="{sx:.2}" y1="{:.2}" x2="{sx:.2}" y2="{:.2}" stroke="#333" stroke-width="1"/>"##, H - MB, H - MB + 5.0).unwrap();
        writeln!(s, r##"<text x="{sx:.2}" y="{ty:.2}">{}</text>"##, fmt_tick(t)).unwrap();
    }
    writeln!(s, r##"<text x="{:.2}" y="{:.2}" font-size="13" font-style="italic">{var}</text>"##,
        ML + cw / 2.0, H - 6.0).unwrap();
    writeln!(s, "</g>").unwrap();

    // ── y-axis ticks & labels ─────────────────────────────────
    writeln!(s, r##"<g font-size="11" font-family="monospace" fill="#333" text-anchor="end">"##).unwrap();
    for t in nice_ticks(y_min, y_max, 6) {
        let sy = py(t, y_min, y_max);
        writeln!(s, r##"<line x1="{:.2}" y1="{sy:.2}" x2="{ML}" y2="{sy:.2}" stroke="#333" stroke-width="1"/>"##, ML - 5.0).unwrap();
        writeln!(s, r##"<text x="{:.2}" y="{:.2}">{}</text>"##, ML - 8.0, sy + 4.0, fmt_tick(t)).unwrap();
    }
    writeln!(s, "</g>").unwrap();

    // ── title ─────────────────────────────────────────────────
    let title = match kind {
        PlotKind::Integrate    => format!("∫({}) d{} ≈ {:.6}", expr_str, var, numerical),
        PlotKind::Differentiate => format!("d/d{}({}) — symbolic derivative", var, expr_str),
    };
    writeln!(s, r##"<text x="{:.2}" y="24" font-size="13" font-family="sans-serif" fill="#222" text-anchor="middle" font-weight="bold">{title}</text>"##,
        ML + cw / 2.0).unwrap();

    // ── legend ────────────────────────────────────────────────
    let lx = W - MR - 160.0;
    let ly = MT + 10.0;
    let lh = if sec_pts.is_some() { 44.0 } else { 24.0 };
    writeln!(s, r##"<rect x="{lx:.2}" y="{ly:.2}" width="155" height="{lh:.2}" fill="white" fill-opacity="0.85" stroke="#ccc" stroke-width="1" rx="4"/>"##).unwrap();
    writeln!(s, r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#1a6bbf" stroke-width="2.5"/>"##,
        lx + 8.0, ly + 14.0, lx + 28.0, ly + 14.0).unwrap();
    writeln!(s, r##"<text x="{:.2}" y="{:.2}" font-size="11" font-family="monospace" fill="#222">f({var}) = {}</text>"##,
        lx + 33.0, ly + 18.0, truncate(expr_str, 14)).unwrap();
    if sec_pts.is_some() {
        let sec_label = match kind {
            PlotKind::Integrate     => format!("F({var}) antiderivative"),
            PlotKind::Differentiate => format!("f'({var}) derivative"),
        };
        writeln!(s, r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#e07020" stroke-width="2" stroke-dasharray="6,4"/>"##,
            lx + 8.0, ly + 34.0, lx + 28.0, ly + 34.0).unwrap();
        writeln!(s, r##"<text x="{:.2}" y="{:.2}" font-size="11" font-family="monospace" fill="#222">{sec_label}</text>"##,
            lx + 33.0, ly + 38.0).unwrap();
    }

    writeln!(s, "</svg>").unwrap();
    s
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}
