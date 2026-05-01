import { math, MAX_EXPRESSION_LENGTH } from './mathInstance.js';

/**
 * Solve f(x) = 0 numerically using Newton's method.
 * Tries multiple starting points to find different roots.
 * @param {string} expression - Expression equal to 0, e.g. "x^2 - 4"
 * @param {string} variable - Variable name (default: 'x')
 * @param {number} initialGuess - Starting point (default: 1)
 * @returns {number[]} Array of found roots
 */
export function solveEquation(expression, variable = 'x', initialGuess = 1) {
  const f = (val) => {
    const scope = { [variable]: val };
    return math.evaluate(expression, scope);
  };

  const df = (val) => {
    const h = 1e-7;
    return (f(val + h) - f(val - h)) / (2 * h);
  };

  const newtonRoot = (start) => {
    let x = start;
    for (let i = 0; i < 1000; i++) {
      const fx = f(x);
      const dfx = df(x);
      if (Math.abs(dfx) < 1e-12) break;
      const xNext = x - fx / dfx;
      if (Math.abs(xNext - x) < 1e-10) {
        return Math.abs(f(xNext)) < 1e-6 ? xNext : null;
      }
      x = xNext;
    }
    return null;
  };

  // Validate the expression eagerly before trying start points so that parse
  // errors and unknown-function errors are surfaced immediately rather than
  // being silently swallowed by the per-start-point try/catch below.
  if (expression.length > MAX_EXPRESSION_LENGTH) {
    throw new Error(`Expression is too long (max ${MAX_EXPRESSION_LENGTH} characters)`);
  }

  try {
    math.parse(expression);
    f(0); // evaluate at a neutral point to catch undefined functions, etc.
  } catch (err) {
    throw new Error(`Invalid expression "${expression}": ${err.message}`);
  }

  const roots = new Set();
  const startPoints = [initialGuess, -initialGuess, 0, 2, -2, 5, -5, 10, -10, 0.5, -0.5];

  for (const start of startPoints) {
    try {
      const root = newtonRoot(start);
      if (root !== null && isFinite(root)) {
        // Round to avoid near-duplicates
        const rounded = Math.round(root * 1e8) / 1e8;
        // Check if already found a very close root
        let isDuplicate = false;
        for (const r of roots) {
          if (Math.abs(r - rounded) < 1e-6) {
            isDuplicate = true;
            break;
          }
        }
        if (!isDuplicate) roots.add(rounded);
      }
    } catch {
      // Suppress only numerical errors tied to a specific starting point
      // (e.g. domain errors like log of a negative, division by zero at
      // that particular value).  Expression-level errors were already caught
      // above and will not reach here.
    }
  }

  return Array.from(roots).sort((a, b) => a - b);
}

/**
 * Solve a linear equation like "2*x + 3 = 7" for a variable.
 * @param {string} expression - Linear equation with '='
 * @param {string} variable - Variable name (default: 'x')
 * @returns {number} Solution value
 */
export function solveLinear(expression, variable = 'x') {
  const parts = expression.split('=');
  if (parts.length !== 2) {
    throw new Error('Linear equation must contain exactly one "=" sign');
  }
  const lhs = parts[0].trim();
  const rhs = parts[1].trim();
  // Rearrange: lhs - rhs = 0
  const combined = `(${lhs}) - (${rhs})`;
  const roots = solveEquation(combined, variable, 1);
  if (roots.length === 0) {
    throw new Error(`Could not solve linear equation: ${expression}`);
  }
  return roots[0];
}
