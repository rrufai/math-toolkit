function escapeHtml(str) {
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

// Tab navigation
document.querySelectorAll('.tab-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
    document.querySelectorAll('.tab-content').forEach(t => t.classList.remove('active'));
    btn.classList.add('active');
    document.getElementById(`tab-${btn.dataset.tab}`).classList.add('active');
  });
});

function showResult(el, content, isError = false) {
  el.classList.remove('hidden', 'error', 'success');
  el.classList.add(isError ? 'error' : 'success');
  el.innerHTML = content;
}

async function post(endpoint, body) {
  const res = await fetch(`/api/${endpoint}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  const data = await res.json();
  if (!res.ok) throw new Error(data.error || 'Request failed');
  return data;
}

async function runEvaluate() {
  const expr = document.getElementById('eval-expr').value.trim();
  const scopeRaw = document.getElementById('eval-scope').value.trim();
  const resultEl = document.getElementById('eval-result');
  if (!expr) { showResult(resultEl, 'Please enter an expression.', true); return; }
  try {
    const scope = scopeRaw ? JSON.parse(scopeRaw) : {};
    const data = await post('evaluate', { expression: expr, scope });
    showResult(resultEl, `<div class="result-label">Result</div><div class="result-value">${escapeHtml(data.result)}</div>`);
  } catch (e) {
    showResult(resultEl, e.message, true);
  }
}

async function runSolve() {
  const eq = document.getElementById('solve-eq').value.trim();
  const variable = document.getElementById('solve-var').value.trim() || 'x';
  const resultEl = document.getElementById('solve-result');
  if (!eq) { showResult(resultEl, 'Please enter an equation.', true); return; }
  try {
    const data = await post('solve', { equation: eq, variable });
    const roots = data.roots.length > 0 ? data.roots.join(', ') : 'No roots found';
    showResult(resultEl, `<div class="result-label">Roots of ${escapeHtml(eq)} = 0</div><div class="result-value">${escapeHtml(roots)}</div>`);
  } catch (e) {
    showResult(resultEl, e.message, true);
  }
}

async function runDerivative() {
  const expr = document.getElementById('deriv-expr').value.trim();
  const variable = document.getElementById('deriv-var').value.trim() || 'x';
  const pointRaw = document.getElementById('deriv-point').value.trim();
  const resultEl = document.getElementById('deriv-result');
  if (!expr) { showResult(resultEl, 'Please enter an expression.', true); return; }
  try {
    const body = { expression: expr, variable };
    if (pointRaw !== '') body.point = parseFloat(pointRaw);
    const data = await post('derivative', body);
    let html = `<div class="result-label">d/d${escapeHtml(variable)} of ${escapeHtml(expr)}</div><div class="result-value">${escapeHtml(data.derivative)}</div>`;
    if (data.valueAtPoint !== undefined) {
      html += `<div class="result-secondary">At ${escapeHtml(variable)} = ${escapeHtml(pointRaw)}: <strong>${escapeHtml(String(data.valueAtPoint))}</strong></div>`;
    }
    showResult(resultEl, html);
  } catch (e) {
    showResult(resultEl, e.message, true);
  }
}

async function runIntegral() {
  const expr = document.getElementById('integ-expr').value.trim();
  const variable = document.getElementById('integ-var').value.trim() || 'x';
  const lower = document.getElementById('integ-lower').value.trim();
  const upper = document.getElementById('integ-upper').value.trim();
  const resultEl = document.getElementById('integ-result');
  if (!expr || lower === '' || upper === '') {
    showResult(resultEl, 'Please fill in all fields.', true); return;
  }
  try {
    const data = await post('integrate', { expression: expr, variable, lower: parseFloat(lower), upper: parseFloat(upper) });
    showResult(resultEl, `<div class="result-label">∫ ${escapeHtml(expr)} d${escapeHtml(variable)} from ${escapeHtml(lower)} to ${escapeHtml(upper)}</div><div class="result-value">${escapeHtml(String(data.result))}</div>`);
  } catch (e) {
    showResult(resultEl, e.message, true);
  }
}

let chartInstance = null;

async function runPlot() {
  const expr = document.getElementById('plot-expr').value.trim();
  const from = parseFloat(document.getElementById('plot-from').value) || -10;
  const to = parseFloat(document.getElementById('plot-to').value) || 10;
  const resultEl = document.getElementById('plot-result');
  if (!expr) { showResult(resultEl, 'Please enter an expression.', true); return; }
  try {
    const data = await post('plot', { expression: expr, from, to, steps: 200 });
    resultEl.classList.remove('hidden', 'error');
    resultEl.classList.add('success');

    const labels = data.points.map(p => p.x.toFixed(2));
    const values = data.points.map(p => p.y);

    if (chartInstance) chartInstance.destroy();
    const ctx = document.getElementById('plot-canvas').getContext('2d');
    chartInstance = new Chart(ctx, {
      type: 'line',
      data: {
        labels,
        datasets: [{
          label: `f(x) = ${escapeHtml(expr)}`,
          data: values,
          borderColor: '#1a56db',
          backgroundColor: 'rgba(26, 86, 219, 0.08)',
          borderWidth: 2,
          pointRadius: 0,
          tension: 0.3,
          fill: true,
        }]
      },
      options: {
        responsive: true,
        plugins: {
          legend: { display: true, position: 'top' },
          tooltip: { mode: 'index', intersect: false }
        },
        scales: {
          x: { ticks: { maxTicksLimit: 10 }, grid: { color: '#e2e8f0' } },
          y: { grid: { color: '#e2e8f0' } }
        }
      }
    });
  } catch (e) {
    showResult(resultEl, e.message, true);
  }
}
