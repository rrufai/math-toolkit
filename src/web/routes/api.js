import { Router } from 'express';
import { evaluate } from '../../math/evaluator.js';
import { solveEquation } from '../../math/solver.js';
import { derivative, integrate } from '../../math/calculus.js';

const apiRouter = Router();
const MAX_EXPRESSION_LENGTH = 500;
const MAX_SCOPE_KEYS = 20;
const MAX_SCOPE_DEPTH = 5;
const MAX_SCOPE_STRING_LENGTH = 200;
const BLOCKED_EXPRESSION_PATTERN = /\b(?:import|createUnit|reviver|evaluate|parse|simplify|derivative|resolve)\s*\(/;

function isPlainObject(value) {
  return Object.prototype.toString.call(value) === '[object Object]';
}

function validateScopeValue(value, depth = 0) {
  if (depth > MAX_SCOPE_DEPTH) {
    throw new Error('scope is too deeply nested');
  }

  if (
    value === null ||
    typeof value === 'number' ||
    typeof value === 'boolean'
  ) {
    if (typeof value === 'number' && !Number.isFinite(value)) {
      throw new Error('scope contains a non-finite number');
    }
    return;
  }

  if (typeof value === 'string') {
    if (value.length > MAX_SCOPE_STRING_LENGTH) {
      throw new Error('scope contains a string value that is too long');
    }
    return;
  }

  if (Array.isArray(value)) {
    if (value.length > MAX_SCOPE_KEYS) {
      throw new Error('scope contains an array that is too large');
    }
    for (const item of value) {
      validateScopeValue(item, depth + 1);
    }
    return;
  }

  if (isPlainObject(value)) {
    const entries = Object.entries(value);
    if (entries.length > MAX_SCOPE_KEYS) {
      throw new Error('scope contains too many keys');
    }
    for (const [key, nestedValue] of entries) {
      if (key.length > 100) {
        throw new Error('scope contains a key that is too long');
      }
      validateScopeValue(nestedValue, depth + 1);
    }
    return;
  }

  throw new Error('scope contains an unsupported value type');
}

function validateEvaluateInput(expression, scope) {
  if (typeof expression !== 'string' || expression.trim() === '') {
    throw new Error('expression is required');
  }

  if (expression.length > MAX_EXPRESSION_LENGTH) {
    throw new Error('expression is too long');
  }

  if (BLOCKED_EXPRESSION_PATTERN.test(expression)) {
    throw new Error('expression contains a disabled function');
  }

  if (scope === null || scope === undefined) {
    return;
  }

  if (!isPlainObject(scope)) {
    throw new Error('scope must be a plain object');
  }

  const entries = Object.entries(scope);
  if (entries.length > MAX_SCOPE_KEYS) {
    throw new Error('scope contains too many keys');
  }

  for (const [key, value] of entries) {
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(key)) {
      throw new Error('scope contains an invalid variable name');
    }
    validateScopeValue(value, 1);
  }
}

apiRouter.post('/evaluate', (req, res) => {
  try {
    const { expression, scope = {} } = req.body;
    validateEvaluateInput(expression, scope);
    const result = evaluate(expression, scope);
    res.json({ result: result.toString() });
  } catch (err) {
    res.status(400).json({ error: err.message });
  }
});

apiRouter.post('/solve', (req, res) => {
  try {
    const { equation, variable = 'x' } = req.body;
    if (!equation) return res.status(400).json({ error: 'equation is required' });
    const roots = solveEquation(equation, variable);
    res.json({ roots });
  } catch (err) {
    res.status(400).json({ error: err.message });
  }
});

apiRouter.post('/derivative', (req, res) => {
  try {
    const { expression, variable = 'x', point } = req.body;
    if (!expression) return res.status(400).json({ error: 'expression is required' });
    let scope = null;
    if (point !== undefined) {
      const parsedPoint = parseFloat(point);
      if (!Number.isFinite(parsedPoint)) {
        return res.status(400).json({ error: 'point must be a finite number' });
      }
      scope = { [variable]: parsedPoint };
    }
    const result = derivative(expression, variable, scope);
    res.json(result);
  } catch (err) {
    res.status(400).json({ error: err.message });
  }
});

apiRouter.post('/integrate', (req, res) => {
  try {
    const { expression, variable = 'x', lower, upper } = req.body;
    if (!expression) return res.status(400).json({ error: 'expression is required' });
    if (lower === undefined || upper === undefined) {
      return res.status(400).json({ error: 'lower and upper bounds are required' });
    }
    const parsedLower = parseFloat(lower);
    const parsedUpper = parseFloat(upper);
    if (!Number.isFinite(parsedLower) || !Number.isFinite(parsedUpper)) {
      return res.status(400).json({ error: 'lower and upper bounds must be finite numbers' });
    }
    const result = integrate(expression, variable, parsedLower, parsedUpper);
    res.json({ result });
  } catch (err) {
    res.status(400).json({ error: err.message });
  }
});

const MAX_PLOT_STEPS = 10000;

apiRouter.post('/plot', (req, res) => {
  try {
    const { expression, from = -10, to = 10, steps = 100, variable = 'x' } = req.body;
    if (!expression) return res.status(400).json({ error: 'expression is required' });

    const plotFrom = Number(from);
    const plotTo = Number(to);
    const plotSteps = Number(steps);

    if (!Number.isFinite(plotFrom) || !Number.isFinite(plotTo)) {
      return res.status(400).json({ error: 'from and to must be finite numbers' });
    }

    if (!Number.isFinite(plotSteps) || !Number.isInteger(plotSteps) || plotSteps < 2) {
      return res.status(400).json({ error: 'steps must be a finite integer greater than or equal to 2' });
    }

    if (plotSteps > MAX_PLOT_STEPS) {
      return res.status(400).json({ error: `steps must not exceed ${MAX_PLOT_STEPS}` });
    }

    const points = [];
    const step = (plotTo - plotFrom) / (plotSteps - 1);
    for (let i = 0; i < plotSteps; i++) {
      const x = plotFrom + i * step;
      try {
        const y = evaluate(expression, { [variable]: x });
        points.push({ x, y: Number.isFinite(y) ? y : null });
      } catch {
        points.push({ x, y: null });
      }
    }
    res.json({ points });
  } catch (err) {
    res.status(400).json({ error: err.message });
  }
});

export { apiRouter };
