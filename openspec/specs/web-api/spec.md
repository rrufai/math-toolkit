## ADDED Requirements

### Requirement: Server configuration
The web server SHALL listen on port `5150`. Configuration is loaded from `config/development.yaml` resolved relative to the process working directory.

#### Scenario: Default port
- **WHEN** the server is started with `cargo run -p rust-integrator-web -- start`
- **THEN** it accepts connections on port `5150`

---

### Requirement: Routes
The server SHALL expose the following routes:

| Method | Path | Description |
|---|---|---|
| `GET` | `/` | Returns an HTML form for expression input |
| `POST` | `/integrate` | Accepts form submission; returns HTML result page |
| `GET` | `/api/integrate` | JSON integration endpoint |
| `POST` | `/differentiate` | Accepts form submission; returns HTML result page |
| `GET` | `/api/differentiate` | JSON differentiation endpoint |

#### Scenario: GET / returns HTML form
- **WHEN** a `GET /` request is made
- **THEN** the response is `200 OK` with `Content-Type: text/html` containing an HTML form with fields `expr`, `a`, `b`

---

### Requirement: POST /integrate — request
The form SHALL accept `application/x-www-form-urlencoded` with fields:

| Field | Type | Required | Description |
|---|---|---|---|
| `expr` | string | yes | Mathematical expression |
| `a` | string | yes | Lower bound (parsed as f64) |
| `b` | string | yes | Upper bound (parsed as f64) |

#### Scenario: Valid form submission
- **WHEN** `POST /integrate` is called with `expr=x^2&a=0&b=1`
- **THEN** the response is `200 OK` HTML containing integration results

---

### Requirement: POST /integrate — response
A successful `POST /integrate` SHALL return HTML containing:

- The expression (HTML-escaped)
- Symbolic antiderivative (if available) or `"(no closed form found)"`
- Numerical result (10 decimal places)
- Inline SVG plot

Note: ASCII plot is NOT included in the HTML response. It is only available via the CLI.

#### Scenario: Symbolic result in HTML response
- **WHEN** integrating `x^2` over `[0, 3]`
- **THEN** the HTML response contains the antiderivative `x^3/3` and numerical value `9.0000000000`

#### Scenario: No closed form in HTML response
- **WHEN** integrating `sin(x^2)` over `[0, 1]`
- **THEN** the HTML response contains `(no closed form found)` and still includes the numerical value

---

### Requirement: GET /api/integrate — request
Query parameters:

| Parameter | Type | Required | Description |
|---|---|---|---|
| `expr` | string | yes | Mathematical expression (URL-encoded) |
| `a` | f64 | yes | Lower bound (parsed by serde from query string) |
| `b` | f64 | yes | Upper bound (parsed by serde from query string) |

Note: `a` and `b` are parsed by the framework's serde deserialiser, not by application code. If they cannot be parsed as `f64`, the framework returns a `422 Unprocessable Entity` response (not the custom `{"error": ...}` JSON format).

#### Scenario: Valid JSON request
- **WHEN** `GET /api/integrate?expr=x%5E2&a=0&b=1` is called
- **THEN** the response is `200 OK` with `Content-Type: application/json`

---

### Requirement: GET /api/integrate — success response
A successful JSON integration response SHALL have the following shape:

```json
{
  "symbolic": "<antiderivative string or null>",
  "numerical": <float>,
  "svg": "<svg string>"
}
```

- `symbolic` SHALL be `null` when no closed form is found
- `numerical` SHALL always be present
- `svg` SHALL be the full inline SVG string
- There is NO `ascii_plot` field in the JSON response

#### Scenario: Symbolic available
- **WHEN** `GET /api/integrate?expr=x%5E2&a=0&b=1`
- **THEN** `symbolic` is a non-null string and `numerical` is approximately `0.3333`

#### Scenario: No closed form
- **WHEN** `GET /api/integrate?expr=sin(x%5E2)&a=0&b=1`
- **THEN** `symbolic` is `null` and `numerical` is a finite float

---

### Requirement: POST /differentiate — response
A successful `POST /differentiate` SHALL return HTML containing:

- The original expression (HTML-escaped)
- The symbolic derivative
- ASCII plot of `f(x)` and `f'(x)`
- Inline SVG plot with `PlotKind::Differentiate`

#### Scenario: Differentiation HTML result
- **WHEN** `POST /differentiate` with `expr=x^2&a=0&b=1`
- **THEN** the HTML contains the derivative `2*x`

---

### Requirement: GET /api/differentiate — success response
A successful JSON differentiation response SHALL have the following shape:

```json
{
  "derivative": "<derivative string>",
  "svg": "<svg string>"
}
```

#### Scenario: Valid derivative response
- **WHEN** `GET /api/differentiate?expr=x%5E2&a=0&b=1`
- **THEN** `derivative` is `"2*x"` (or equivalent simplified form)

---

### Requirement: Error responses
Error handling differs between HTML (form) and JSON endpoints:

**HTML endpoints (`POST /integrate`, `POST /differentiate`)**:
- Bounds are received as strings and parsed by application code
- Bad bounds → HTML error page containing `"not a valid number"`
- Bad expression → HTML error page with parse error message

**JSON endpoints (`GET /api/integrate`, `GET /api/differentiate`)**:
- Bounds are received as `f64` query parameters parsed by serde
- Bad bounds (non-numeric) → framework-level `422 Unprocessable Entity` (not custom JSON)
- Bad expression → `200 OK` with `{"error": "<parse error message>"}`
- Unsupported differentiation (abs) → `200 OK` with `{"error": "<message>"}`

#### Scenario: HTML error — bad expression
- **WHEN** `POST /integrate` is called with an unparseable expression
- **THEN** the response is `200 OK` HTML containing `Error` and the parse error message

#### Scenario: HTML error — bad bounds
- **WHEN** `POST /integrate` is called with `a=abc`
- **THEN** the response is `200 OK` HTML containing `"not a valid number"`

#### Scenario: JSON error — bad expression
- **WHEN** `GET /api/integrate?expr=sin(x+&a=0&b=1`
- **THEN** the response body is `{"error": "<parse error message>"}` with status `200`

#### Scenario: JSON error — bad bounds (422 from framework)
- **WHEN** `GET /api/integrate?expr=x&a=foo&b=1`
- **THEN** the response status is `422` (framework-level, not application-level)

---

### Requirement: HTML escaping
All user-supplied input rendered into HTML responses SHALL be escaped. The following characters SHALL be replaced:

| Character | Escaped form |
|---|---|
| `&` | `&amp;` |
| `<` | `&lt;` |
| `>` | `&gt;` |
| `"` | `&quot;` |

This applies to: expression strings, error messages, and any user input echoed back in the response.

#### Scenario: XSS prevention
- **WHEN** the expression `<script>alert(1)</script>` is submitted
- **THEN** the HTML response contains `&lt;script&gt;` and does not execute JavaScript
