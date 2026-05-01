## ADDED Requirements

### Requirement: SVG plot dimensions
`render_svg(...)` SHALL produce an SVG document with the following fixed dimensions and margins:

| Property | Value |
|---|---|
| Total width | `600 px` |
| Total height | `400 px` |
| Left margin (`ML`) | `60 px` |
| Right margin (`MR`) | `20 px` |
| Top margin (`MT`) | `45 px` |
| Bottom margin (`MB`) | `50 px` |
| Plot area width | `600 - 60 - 20 = 520 px` |
| Plot area height | `400 - 45 - 50 = 305 px` |
| Sample count | `N+1 = 401` points (iterates `0..=N` where `N=400`) |

#### Scenario: SVG dimensions
- **WHEN** `render_svg` is called with any expression
- **THEN** the output string contains `width="600"` and `height="400"`

---

### Requirement: ASCII plot dimensions
`render_ascii_string(...)` SHALL produce a text grid with the following fixed dimensions:

| Property | Value |
|---|---|
| Columns | `60` |
| Rows | `20` |
| Sample count | `60` points |

#### Scenario: ASCII plot line count
- **WHEN** `render_ascii_string` is called
- **THEN** the returned string contains exactly `20` data rows plus axis/label rows

---

### Requirement: Y-range calculation
Both SVG and ASCII plots SHALL compute the y-range from sampled values as follows:

1. Collect all finite sample values (discard `NaN` and `Inf`)
2. If no finite values exist:
   - SVG: use the default range `[-1.0, 1.0]`
   - ASCII: return the string `"  (no finite values to plot)\n"` immediately
3. If all finite values are equal, the effective span is clamped to a minimum of `1e-9` before padding (prevents zero-division)
4. Apply 5% padding: `y_min = min - 0.05*(max-min)`, `y_max = max + 0.05*(max-min)`

#### Scenario: Normal range with padding
- **WHEN** sampled y-values span `[0.0, 4.0]`
- **THEN** the plot y-axis spans `[-0.2, 4.2]`

#### Scenario: All values infinite — default range (SVG)
- **WHEN** all sampled values are `NaN` or `Inf` and rendering SVG
- **THEN** the y-axis defaults to `[-1.0, 1.0]`

#### Scenario: All values infinite — ASCII early return
- **WHEN** all sampled values are `NaN` or `Inf` and rendering ASCII
- **THEN** the function returns `"  (no finite values to plot)\n"`

---

### Requirement: NaN and Inf handling
Both plot renderers SHALL skip non-finite sample values. In SVG plots, the polyline SHALL also be broken when a y-value falls outside `[y_min, y_max]` (out-of-range values are not clipped — they break the line). This produces multiple `<polyline>` segments rather than one continuous line.

#### Scenario: Discontinuous function
- **WHEN** plotting `tan(x)` near its asymptote
- **THEN** the SVG contains multiple polyline segments separated at the discontinuity

#### Scenario: Single NaN value does not crash
- **WHEN** a single sampled value is `NaN`
- **THEN** rendering completes without error

---

### Requirement: Zero-line
Both renderers SHALL draw a horizontal line at y=0 if and only if `y_min < 0` and `y_max > 0`.

#### Scenario: Zero-line shown when range crosses zero
- **WHEN** the y-range is `[-2.0, 2.0]`
- **THEN** a zero axis line is rendered

#### Scenario: Zero-line omitted when all values positive
- **WHEN** the y-range is `[1.0, 5.0]`
- **THEN** no zero axis line is rendered

---

### Requirement: Tick formatting
SVG and ASCII renderers use different tick formatters:

**SVG axis ticks** (`fmt_tick`):
| Condition | Format |
|---|---|
| `v == 0.0` | `"0"` |
| `|v| >= 1000` or `0 < |v| < 0.01` | `"{:.2e}"` (2 decimal places, e.g. `1.23e4`) |
| `(v - round(v)).abs() < 1e-9 * max(|v|, 1)` (integer) | `"{}"` as `i64` (e.g. `3`) |
| Otherwise | `"{:.2}"` (2 decimal places) |

**ASCII y-axis labels** (`fmt_y`):
| Condition | Format |
|---|---|
| `|v| >= 1000` or `0 < |v| < 0.01` | `"{:.1e}"` (1 decimal place) |
| Otherwise | `"{:.3}"` (3 decimal places) |

Both use the same condition for scientific notation (`|v| >= 1000` or `0 < |v| < 0.01`).

#### Scenario: SVG integer tick
- **WHEN** an SVG tick value is `2.0`
- **THEN** it is displayed as `2`

#### Scenario: SVG large value scientific notation
- **WHEN** an SVG tick value is `12345.0`
- **THEN** it is displayed as `1.23e4` (2 decimal places in mantissa)

#### Scenario: ASCII tick format
- **WHEN** an ASCII y-axis label value is `0.123`
- **THEN** it is displayed as `0.123` (3 decimal places)

---

### Requirement: Plot kinds
The `PlotKind` enum controls plot appearance:

| `PlotKind` | Behaviour |
|---|---|
| `Integrate` | Shows `f(x)` and `F(x)` (antiderivative); area under `f(x)` is shaded; legend shows `F(x)` |
| `Differentiate` | Shows `f(x)` and `f'(x)` (derivative); no area shading; legend shows `f'(x)` |

#### Scenario: Integration plot has area shading
- **WHEN** `render_svg` is called with `PlotKind::Integrate`
- **THEN** the SVG output contains a filled area element under `f(x)`

#### Scenario: Differentiation plot has no area shading
- **WHEN** `render_svg` is called with `PlotKind::Differentiate`
- **THEN** the SVG output contains no filled area element

---

### Requirement: SVG is self-contained
`render_svg` SHALL return a complete SVG string suitable for direct embedding in HTML (`<div>` or `<figure>`). It MUST NOT reference external files or require separate stylesheets.

#### Scenario: Inline embedding
- **WHEN** the SVG string is inserted directly into an HTML document
- **THEN** it renders correctly without external dependencies
