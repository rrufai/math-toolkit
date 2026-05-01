## ADDED Requirements

### Requirement: /solve routes
The server SHALL expose two new routes for equation solving:

| Method | Path | Description |
|---|---|---|
| `POST` | `/solve` | HTML form submission; returns HTML result page |
| `GET` | `/api/solve` | JSON endpoint |

#### Scenario: POST /solve route exists
- **WHEN** `POST /solve` is called with a valid form
- **THEN** the response is `200 OK` HTML (not `404`)

#### Scenario: GET /api/solve route exists
- **WHEN** `GET /api/solve?equation=x%5E2-2&a=1&b=2` is called
- **THEN** the response is `200 OK` JSON (not `404`)

---

### Requirement: / index page updated
The `GET /` HTML form page SHALL include a Solve section with a form posting to `/solve` with fields `equation`, `a`, `b`.

#### Scenario: Index includes solve form
- **WHEN** `GET /` is requested
- **THEN** the HTML contains `action="/solve"` and an input named `equation`

---

### Requirement: POST /solve — request
The form SHALL accept `application/x-www-form-urlencoded` with fields:

| Field | Type | Required | Description |
|---|---|---|---|
| `equation` | string | yes | Equation string (`f(x)` or `f(x) = g(x)`) |
| `a` | string | yes | Lower bracket bound (parsed as f64) |
| `b` | string | yes | Upper bracket bound (parsed as f64) |

#### Scenario: Valid form submission
- **WHEN** `POST /solve` is called with `equation=x^2-2&a=1&b=2`
- **THEN** the response is `200 OK` HTML containing the root value

---

### Requirement: POST /solve — response
A successful `POST /solve` SHALL return HTML containing:

- The equation string (HTML-escaped)
- The bracketing interval `[a, b]`
- The root value (10 decimal places)
- The residual `|f(root)|`
- The iteration count
- Inline SVG plot of `f(x)` over `[a, b]`

#### Scenario: Root displayed in HTML
- **WHEN** `POST /solve` with `equation=x^2-2&a=1&b=2`
- **THEN** HTML contains the root value `1.4142135623...`

---

### Requirement: GET /api/solve — request
Query parameters:

| Parameter | Type | Required | Description |
|---|---|---|---|
| `equation` | string | yes | Equation string (URL-encoded) |
| `a` | f64 | yes | Lower bracket bound (parsed by serde) |
| `b` | f64 | yes | Upper bracket bound (parsed by serde) |

Note: `a` and `b` are parsed by serde. Non-numeric values produce a framework-level `422`.

#### Scenario: Valid JSON request
- **WHEN** `GET /api/solve?equation=x%5E2-2&a=1&b=2` is called
- **THEN** the response is `200 OK` with `Content-Type: application/json`

---

### Requirement: GET /api/solve — success response
A successful JSON solve response SHALL have the following shape:

```json
{
  "root": <float>,
  "residual": <float>,
  "iterations": <integer>,
  "svg": "<svg string>"
}
```

#### Scenario: Root in JSON response
- **WHEN** `GET /api/solve?equation=x%5E2-2&a=1&b=2`
- **THEN** `root` is within `1e-10` of `√2` and `residual < 1e-10`

#### Scenario: No sign change — JSON error
- **WHEN** `GET /api/solve?equation=x%5E2&a=1&b=3`
- **THEN** response is `200 OK` with `{"error": "no sign change..."}`

---

### Requirement: /solve error responses
Error handling SHALL follow the same pattern as existing endpoints:

| Error type | HTML endpoint | JSON endpoint |
|---|---|---|
| No sign change | HTML error page | `{"error": "..."}` with status `200` |
| Parse error | HTML error page | `{"error": "..."}` with status `200` |
| Bad bounds (non-numeric) | HTML error page | Framework `422` |

#### Scenario: HTML error — no sign change
- **WHEN** `POST /solve` is called with an equation that has no root in `[a, b]`
- **THEN** the HTML response contains `Error` and the no-sign-change message

#### Scenario: JSON error — parse error
- **WHEN** `GET /api/solve?equation=sin(x+&a=0&b=4`
- **THEN** response body is `{"error": "<parse error>"}` with status `200`
