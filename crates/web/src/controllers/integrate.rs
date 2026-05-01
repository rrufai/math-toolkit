use axum::{extract::Query as AxumQuery, response::Response, Form};
use loco_rs::{
    controller::{format, Routes},
    Result,
};
use serde::{Deserialize, Serialize};

use integrator_core::{integrator, plot, parse};
use solver_core::solve;

pub fn routes() -> Routes {
    Routes::new()
        .add("/", axum::routing::get(index))
        .add("/integrate", axum::routing::post(handle_integrate))
        .add("/api/integrate", axum::routing::get(api_integrate))
        .add("/differentiate", axum::routing::post(handle_differentiate))
        .add("/api/differentiate", axum::routing::get(api_differentiate))
        .add("/solve", axum::routing::post(handle_solve))
        .add("/api/solve", axum::routing::get(api_solve))
}

// ── Request / response types ─────────────────────────────────

#[derive(Deserialize)]
struct IntegrateForm {
    expr: String,
    a: String,
    b: String,
}

#[derive(Deserialize)]
struct IntegrateQuery {
    expr: String,
    a: f64,
    b: f64,
}

#[derive(Serialize)]
struct ApiResponse {
    symbolic: Option<String>,
    numerical: f64,
    svg: String,
}

struct ComputedResult {
    symbolic: Option<String>,
    numerical: f64,
    svg: String,
}

// ── Core computation ─────────────────────────────────────────

fn compute_result(expr_str: &str, a: f64, b: f64) -> std::result::Result<ComputedResult, String> {
    let ast = parse(expr_str)?;
    let var = integrator_core::first_var(&ast).unwrap_or_else(|| "x".to_string());
    let symbolic = integrator::integrate_symbolic(&ast)
        .ok()
        .map(|a| a.to_string_repr());
    let numerical = integrator::integrate_numerical(&|x| ast.eval(x), a, b);
    let anti_fn: Option<Box<dyn Fn(f64) -> f64>> = integrator::integrate_symbolic(&ast)
        .ok()
        .map(|anti| Box::new(move |x| anti.eval(x)) as Box<dyn Fn(f64) -> f64>);
    let svg = plot::render_svg(
        expr_str,
        &var,
        a,
        b,
        &|x| ast.eval(x),
        anti_fn.as_deref(),
        numerical,
    );
    Ok(ComputedResult {
        symbolic,
        numerical,
        svg,
    })
}

#[derive(Deserialize)]
struct DifferentiateForm {
    expr: String,
    a: String,
    b: String,
}

#[derive(Deserialize)]
struct DifferentiateQuery {
    expr: String,
    a: f64,
    b: f64,
}

#[derive(Serialize)]
struct DiffApiResponse {
    derivative: String,
    svg: String,
}

struct DiffComputedResult {
    derivative: String,
    svg: String,
}

// ── Solve types ───────────────────────────────────────────────

#[derive(Deserialize)]
struct SolveForm {
    equation: String,
    a: String,
    b: String,
}

#[derive(Deserialize)]
struct SolveQuery {
    equation: String,
    a: f64,
    b: f64,
}

#[derive(Serialize)]
struct SolveApiResponse {
    root: f64,
    residual: f64,
    iterations: u32,
    svg: String,
}

struct SolveComputedResult {
    root: f64,
    residual: f64,
    iterations: u32,
    svg: String,
}

fn compute_diff_result(expr_str: &str, a: f64, b: f64) -> std::result::Result<DiffComputedResult, String> {
    let ast = parse(expr_str)?;
    let var = integrator_core::first_var(&ast).unwrap_or_else(|| "x".to_string());
    let deriv_ast = integrator::differentiate_symbolic(&ast)?;
    let derivative = deriv_ast.to_string_repr();
    let svg = plot::render_svg_diff(
        expr_str,
        &var,
        a,
        b,
        &|x| ast.eval(x),
        Some(&|x| deriv_ast.eval(x)),
    );
    Ok(DiffComputedResult { derivative, svg })
}

fn compute_solve_result(equation: &str, a: f64, b: f64) -> std::result::Result<SolveComputedResult, String> {
    let r = solve(equation, a, b)?;

    // Build the plot expression (lhs - rhs for two-sided equations).
    let plot_expr = if equation.contains('=') {
        let parts: Vec<&str> = equation.splitn(2, '=').collect();
        format!("({}) - ({})", parts[0].trim(), parts[1].trim())
    } else {
        equation.to_string()
    };

    let ast = parse(&plot_expr)?;
    let var = integrator_core::first_var(&ast).unwrap_or_else(|| "x".to_string());
    let svg = plot::render_svg(
        &plot_expr,
        &var,
        a,
        b,
        &|x| ast.eval(x),
        None,
        r.residual,
    );
    Ok(SolveComputedResult {
        root: r.root,
        residual: r.residual,
        iterations: r.iterations,
        svg,
    })
}

// ── HTML helpers ─────────────────────────────────────────────

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Rust Integrator</title>
  <style>
    body { font-family: sans-serif; max-width: 700px; margin: 40px auto; padding: 0 20px; }
    input { margin: 4px; padding: 6px 10px; font-size: 1em; }
    button { padding: 7px 18px; font-size: 1em; cursor: pointer; }
    label { font-weight: bold; }
    h2 { margin-top: 32px; }
  </style>
</head>
<body>
  <h1>∫ Rust Integrator / Differentiator</h1>

  <h2>Integrate</h2>
  <p>Enter a mathematical expression and bounds to integrate symbolically and numerically.</p>
  <form method="post" action="/integrate">
    <div><label>f(x) = </label><input name="expr" type="text" size="30" placeholder="x^2 + sin(x)" required></div>
    <div><label>a = </label><input name="a" type="text" size="10" value="0" required></div>
    <div><label>b = </label><input name="b" type="text" size="10" value="1" required></div>
    <div><button type="submit">Integrate</button></div>
  </form>

  <h2>Differentiate</h2>
  <p>Enter a mathematical expression to differentiate symbolically.</p>
  <form method="post" action="/differentiate">
    <div><label>f(x) = </label><input name="expr" type="text" size="30" placeholder="x^3 + sin(x)" required></div>
    <div><label>a = </label><input name="a" type="text" size="10" value="0" required></div>
    <div><label>b = </label><input name="b" type="text" size="10" value="2" required></div>
    <div><button type="submit">Differentiate</button></div>
  </form>

  <h2>Solve</h2>
  <p>Find a root of f(x) = 0 or f(x) = g(x) within the interval [a, b].</p>
  <form method="post" action="/solve">
    <div><label>equation = </label><input name="equation" type="text" size="30" placeholder="x^2 - 2" required></div>
    <div><label>a = </label><input name="a" type="text" size="10" value="1" required></div>
    <div><label>b = </label><input name="b" type="text" size="10" value="2" required></div>
    <div><button type="submit">Solve</button></div>
  </form>

  <hr>
  <p>JSON APIs:<br>
    <code>GET /api/integrate?expr=x^2&amp;a=0&amp;b=3</code><br>
    <code>GET /api/differentiate?expr=x^3&amp;a=0&amp;b=2</code><br>
    <code>GET /api/solve?equation=x^2-2&amp;a=1&amp;b=2</code>
  </p>
</body>
</html>"#;

fn result_page(expr: &str, a: f64, b: f64, res: &ComputedResult) -> String {
    let sym_line = match &res.symbolic {
        Some(s) => format!("<p><b>∫f(x)dx</b> = {} + C</p>", html_escape(s)),
        None => "<p><b>∫f(x)dx</b> = (no closed form found)</p>".to_string(),
    };
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Integration Result</title>
  <style>
    body {{ font-family: sans-serif; max-width: 800px; margin: 40px auto; padding: 0 20px; }}
    pre {{ background: #f4f4f4; padding: 12px; overflow-x: auto; font-size: 0.9em; }}
    .result {{ border: 1px solid #ccc; border-radius: 6px; padding: 16px; margin-bottom: 20px; }}
  </style>
</head>
<body>
  <h1>Integration Result</h1>
  <div class="result">
    <p><b>f(x)</b> = {expr_escaped}</p>
    <p><b>Interval</b>: [{a}, {b}]</p>
    {sym_line}
    <p><b>Numerical ≈</b> {numerical:.10}</p>
  </div>
  <h2>SVG Plot</h2>
  {svg}
  <p><a href="/">← Back</a></p>
</body>
</html>"#,
        expr_escaped = html_escape(expr),
        a = a,
        b = b,
        sym_line = sym_line,
        numerical = res.numerical,
        svg = res.svg,
    )
}

fn diff_result_page(expr: &str, a: f64, b: f64, res: &DiffComputedResult) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Differentiation Result</title>
  <style>
    body {{ font-family: sans-serif; max-width: 800px; margin: 40px auto; padding: 0 20px; }}
    pre {{ background: #f4f4f4; padding: 12px; overflow-x: auto; font-size: 0.9em; }}
    .result {{ border: 1px solid #ccc; border-radius: 6px; padding: 16px; margin-bottom: 20px; }}
  </style>
</head>
<body>
  <h1>Differentiation Result</h1>
  <div class="result">
    <p><b>f(x)</b> = {expr_escaped}</p>
    <p><b>Plot interval</b>: [{a}, {b}]</p>
    <p><b>f'(x)</b> = {deriv_escaped}</p>
  </div>
  <h2>SVG Plot</h2>
  {svg}
  <p><a href="/">← Back</a></p>
</body>
</html>"#,
        expr_escaped = html_escape(expr),
        a = a,
        b = b,
        deriv_escaped = html_escape(&res.derivative),
        svg = res.svg,
    )
}

fn error_page(msg: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>Error</title></head>
<body>
  <h1>Error</h1>
  <p style="color:red">{}</p>
  <p><a href="/">← Back</a></p>
</body>
</html>"#,
        html_escape(msg)
    )
}

fn solve_result_page(equation: &str, a: f64, b: f64, res: &SolveComputedResult) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Solve Result</title>
  <style>
    body {{ font-family: sans-serif; max-width: 800px; margin: 40px auto; padding: 0 20px; }}
    .result {{ border: 1px solid #ccc; border-radius: 6px; padding: 16px; margin-bottom: 20px; }}
  </style>
</head>
<body>
  <h1>Solve Result</h1>
  <div class="result">
    <p><b>equation</b> = {eq_escaped}</p>
    <p><b>Interval</b>: [{a}, {b}]</p>
    <p><b>root ≈</b> {root:.10}</p>
    <p><b>|f(root)| ≈</b> {residual:e}</p>
    <p><b>iterations</b> = {iterations}</p>
  </div>
  <h2>SVG Plot</h2>
  {svg}
  <p><a href="/">← Back</a></p>
</body>
</html>"#,
        eq_escaped = html_escape(equation),
        a = a,
        b = b,
        root = res.root,
        residual = res.residual,
        iterations = res.iterations,
        svg = res.svg,
    )
}

// ── Plain axum Router for testing (no AppContext state needed) ──

#[cfg(test)]
pub fn test_router() -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::get(index))
        .route("/integrate", axum::routing::post(handle_integrate))
        .route("/api/integrate", axum::routing::get(api_integrate))
        .route("/differentiate", axum::routing::post(handle_differentiate))
        .route("/api/differentiate", axum::routing::get(api_differentiate))
        .route("/solve", axum::routing::post(handle_solve))
        .route("/api/solve", axum::routing::get(api_solve))
}

// ── Route handlers ────────────────────────────────────────────

async fn index() -> Result<Response> {
    format::html(INDEX_HTML)
}

async fn handle_integrate(
    Form(form): Form<IntegrateForm>,
) -> Result<Response> {
    let a = match form.a.parse::<f64>() {
        Ok(v) => v,
        Err(_) => return format::html(&error_page(&format!("'{}' is not a valid number", form.a))),
    };
    let b = match form.b.parse::<f64>() {
        Ok(v) => v,
        Err(_) => return format::html(&error_page(&format!("'{}' is not a valid number", form.b))),
    };
    match compute_result(&form.expr, a, b) {
        Ok(res) => format::html(&result_page(&form.expr, a, b, &res)),
        Err(e) => format::html(&error_page(&e)),
    }
}

async fn api_integrate(
    AxumQuery(q): AxumQuery<IntegrateQuery>,
) -> Result<Response> {
    match compute_result(&q.expr, q.a, q.b) {
        Ok(res) => format::json(ApiResponse {
            symbolic: res.symbolic,
            numerical: res.numerical,
            svg: res.svg,
        }),
        Err(e) => {
            let body = serde_json::json!({ "error": e });
            format::json(body)
        }
    }
}

async fn handle_differentiate(
    Form(form): Form<DifferentiateForm>,
) -> Result<Response> {
    let a = match form.a.parse::<f64>() {
        Ok(v) => v,
        Err(_) => return format::html(&error_page(&format!("'{}' is not a valid number", form.a))),
    };
    let b = match form.b.parse::<f64>() {
        Ok(v) => v,
        Err(_) => return format::html(&error_page(&format!("'{}' is not a valid number", form.b))),
    };
    match compute_diff_result(&form.expr, a, b) {
        Ok(res) => format::html(&diff_result_page(&form.expr, a, b, &res)),
        Err(e) => format::html(&error_page(&e)),
    }
}

async fn api_differentiate(
    AxumQuery(q): AxumQuery<DifferentiateQuery>,
) -> Result<Response> {
    match compute_diff_result(&q.expr, q.a, q.b) {
        Ok(res) => format::json(DiffApiResponse {
            derivative: res.derivative,
            svg: res.svg,
        }),
        Err(e) => {
            let body = serde_json::json!({ "error": e });
            format::json(body)
        }
    }
}

async fn handle_solve(
    Form(form): Form<SolveForm>,
) -> Result<Response> {
    let a = match form.a.parse::<f64>() {
        Ok(v) => v,
        Err(_) => return format::html(&error_page(&format!("'{}' is not a valid number", form.a))),
    };
    let b = match form.b.parse::<f64>() {
        Ok(v) => v,
        Err(_) => return format::html(&error_page(&format!("'{}' is not a valid number", form.b))),
    };
    match compute_solve_result(&form.equation, a, b) {
        Ok(res) => format::html(&solve_result_page(&form.equation, a, b, &res)),
        Err(e) => format::html(&error_page(&e)),
    }
}

async fn api_solve(
    AxumQuery(q): AxumQuery<SolveQuery>,
) -> Result<Response> {
    match compute_solve_result(&q.equation, q.a, q.b) {
        Ok(res) => format::json(SolveApiResponse {
            root: res.root,
            residual: res.residual,
            iterations: res.iterations,
            svg: res.svg,
        }),
        Err(e) => {
            let body = serde_json::json!({ "error": e });
            format::json(body)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt; // for `oneshot`

    async fn body_string(body: Body) -> String {
        let bytes = body.collect().await.unwrap().to_bytes();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    // ── GET / ─────────────────────────────────────────────────

    #[tokio::test]
    async fn test_index_returns_form() {
        let app = test_router();
        let resp = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp.into_body()).await;
        assert!(body.contains("<form"));
        assert!(body.contains("Rust Integrator"));
        assert!(body.contains(r#"action="/integrate""#));
    }

    // ── POST /integrate — happy path ──────────────────────────

    #[tokio::test]
    async fn test_post_integrate_x_squared() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/integrate")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("expr=x%5E2&a=0&b=3"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp.into_body()).await;
        assert!(body.contains("x^3/3"));
        assert!(body.contains("9.0000000000"));
        assert!(body.contains("<svg"));
    }

    #[tokio::test]
    async fn test_post_integrate_sin() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/integrate")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("expr=sin(x)&a=0&b=3.14159265358979"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp.into_body()).await;
        assert!(body.contains("-cos(x)"));
        assert!(body.contains("2.0000000000"));
    }

    #[tokio::test]
    async fn test_post_integrate_no_closed_form() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/integrate")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("expr=sin(x%5E2)&a=0&b=1"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp.into_body()).await;
        assert!(body.contains("no closed form found"));
    }

    // ── POST /integrate — error paths ────────────────────────

    #[tokio::test]
    async fn test_post_integrate_bad_expr() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/integrate")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("expr=sin(x+&a=0&b=1"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp.into_body()).await;
        assert!(body.contains("Error") || body.contains("error"));
    }

    #[tokio::test]
    async fn test_post_integrate_bad_bound_a() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/integrate")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("expr=x%5E2&a=abc&b=3"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp.into_body()).await;
        assert!(body.contains("not a valid number"));
    }

    #[tokio::test]
    async fn test_post_integrate_bad_bound_b() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/integrate")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("expr=x%5E2&a=0&b=xyz"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp.into_body()).await;
        assert!(body.contains("not a valid number"));
    }

    // ── GET /api/integrate ────────────────────────────────────

    #[tokio::test]
    async fn test_api_integrate_json() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/integrate?expr=x%5E2&a=0&b=3")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp.into_body()).await;
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["symbolic"].as_str().unwrap(), "x^3/3");
        let num = json["numerical"].as_f64().unwrap();
        assert!((num - 9.0).abs() < 1e-6);
        assert!(json["svg"].as_str().unwrap().contains("<svg"));
    }

    #[tokio::test]
    async fn test_api_integrate_no_closed_form() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/integrate?expr=sin(x%5E2)&a=0&b=1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_str(&body_string(resp.into_body()).await).unwrap();
        assert!(json["symbolic"].is_null());
        assert!(json["numerical"].as_f64().unwrap().is_finite());
    }

    #[tokio::test]
    async fn test_api_integrate_error_json() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/integrate?expr=sin(x+&a=0&b=1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_str(&body_string(resp.into_body()).await).unwrap();
        assert!(json.get("error").is_some());
    }

    // ── compute_diff_result unit tests ───────────────────────

    #[test]
    fn test_compute_diff_result_x_squared() {
        let r = compute_diff_result("x^2", 0.0, 2.0).unwrap();
        // d/dx x^2 = 2x
        let e = integrator_core::parse(&r.derivative).unwrap();
        assert!((e.eval(3.0) - 6.0).abs() < 1e-10);
        assert!(r.svg.contains("<svg"));
    }

    #[test]
    fn test_compute_diff_result_parse_error() {
        assert!(compute_diff_result("sin(x +", 0.0, 1.0).is_err());
    }

    #[test]
    fn test_compute_diff_result_abs_unsupported() {
        assert!(compute_diff_result("abs(x)", 0.0, 1.0).is_err());
    }

    // ── POST /differentiate ───────────────────────────────────

    #[tokio::test]
    async fn test_post_differentiate_x_squared() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/differentiate")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("expr=x%5E2&a=0&b=2"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp.into_body()).await;
        assert!(body.contains("Differentiation Result"));
        assert!(body.contains("<svg"));
    }

    #[tokio::test]
    async fn test_post_differentiate_bad_expr() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/differentiate")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("expr=sin(x+&a=0&b=1"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp.into_body()).await;
        assert!(body.contains("Error") || body.contains("error"));
    }

    // ── GET /api/differentiate ────────────────────────────────

    #[tokio::test]
    async fn test_api_differentiate_json() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/differentiate?expr=x%5E3&a=0&b=2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp.into_body()).await;
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        // d/dx x^3 = 3x^2, at x=2 → 12
        let deriv_str = json["derivative"].as_str().unwrap();
        let e = integrator_core::parse(deriv_str).unwrap();
        assert!((e.eval(2.0) - 12.0).abs() < 1e-6);
        assert!(json["svg"].as_str().unwrap().contains("<svg"));
    }

    #[tokio::test]
    async fn test_api_differentiate_error() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/differentiate?expr=abs(x)&a=0&b=2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_str(&body_string(resp.into_body()).await).unwrap();
        assert!(json.get("error").is_some());
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape(r#"say "hi""#), "say &quot;hi&quot;");
        assert_eq!(html_escape("plain"), "plain");
    }

    // ── Solve route tests ─────────────────────────────────────

    #[tokio::test]
    async fn test_post_solve_route_exists() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/solve")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("equation=x%5E2-2&a=1&b=2"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_api_solve_route_exists() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/solve?equation=x%5E2-2&a=1&b=2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp.into_body()).await;
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(json.get("root").is_some());
    }

    #[tokio::test]
    async fn test_post_solve_contains_root() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/solve")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("equation=x%5E2-2&a=1&b=2"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp.into_body()).await;
        assert!(body.contains("1.414213562"), "body={}", body);
    }

    #[tokio::test]
    async fn test_api_solve_success() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/solve?equation=x%5E2-2&a=1&b=2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp.into_body()).await;
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        let root = json["root"].as_f64().unwrap();
        let residual = json["residual"].as_f64().unwrap();
        assert!((root - 2f64.sqrt()).abs() < 1e-10, "root={}", root);
        assert!(residual < 1e-10, "residual={}", residual);
        assert!(json["svg"].as_str().unwrap().contains("<svg"));
    }

    #[tokio::test]
    async fn test_api_solve_no_sign_change() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/solve?equation=x%5E2&a=1&b=3")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_str(&body_string(resp.into_body()).await).unwrap();
        let err = json["error"].as_str().unwrap();
        assert!(err.contains("no sign change"), "err={}", err);
    }

    #[tokio::test]
    async fn test_api_solve_parse_error() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/solve?equation=sin(x%2B&a=0&b=4")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_str(&body_string(resp.into_body()).await).unwrap();
        assert!(json.get("error").is_some());
    }

    #[tokio::test]
    async fn test_index_contains_solve_form() {
        let app = test_router();
        let resp = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp.into_body()).await;
        assert!(body.contains(r#"action="/solve""#), "body missing solve form");
        assert!(body.contains(r#"name="equation""#), "body missing equation input");
    }

    // ── compute_result unit tests ─────────────────────────────

    #[test]
    fn test_compute_result_symbolic() {
        let r = compute_result("x^2", 0.0, 3.0).unwrap();
        assert_eq!(r.symbolic.as_deref(), Some("x^3/3"));
        assert!((r.numerical - 9.0).abs() < 1e-6);
        assert!(r.svg.contains("<svg"));
    }

    #[test]
    fn test_compute_result_no_symbolic() {
        let r = compute_result("sin(x^2)", 0.0, 1.0).unwrap();
        assert!(r.symbolic.is_none());
        assert!(r.numerical.is_finite());
    }

    #[test]
    fn test_compute_result_parse_error() {
        assert!(compute_result("sin(x +", 0.0, 1.0).is_err());
    }
}
