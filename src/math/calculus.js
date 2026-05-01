import { create, all } from 'mathjs';

const math = create(all);

/**
 * Compute the symbolic derivative of an expression.
 * @param {string} expression - Math expression string
 * @param {string} variable - Variable to differentiate with respect to (default: 'x')
 * @param {Object} scope - Optional scope to evaluate derivative at a point
 * @returns {{ derivative: string, valueAtPoint?: number }}
 */
export function derivative(expression, variable = 'x', scope = null) {
  try {
    const node = math.parse(expression);
    const derivNode = math.derivative(node, variable);
    const derivStr = derivNode.toString();

    const result = { derivative: derivStr };

    if (scope && typeof scope === 'object') {
      result.valueAtPoint = math.evaluate(derivStr, scope);
    }

    return result;
  } catch (err) {
    throw new Error(`Failed to differentiate "${expression}": ${err.message}`);
  }
}

/**
 * Numerically integrate an expression using Simpson's rule.
 * @param {string} expression - Math expression string
 * @param {string} variable - Integration variable (default: 'x')
 * @param {number} lowerBound - Lower bound of integration
 * @param {number} upperBound - Upper bound of integration
 * @param {number} steps - Number of steps (must be even; default: 1000)
 * @returns {number} Numerical integral value
 */
export function integrate(expression, variable = 'x', lowerBound, upperBound, steps = 1000) {
  if (!Number.isFinite(lowerBound) || !Number.isFinite(upperBound)) {
    throw new Error('Lower and upper bounds must be finite numbers');
  }

  if (!Number.isInteger(steps) || steps < 2) {
    throw new Error('Steps must be an integer greater than or equal to 2');
  }

  // Ensure steps is even
  if (steps % 2 !== 0) steps += 1;

  const f = (val) => {
    const scope = { [variable]: val };
    return math.evaluate(expression, scope);
  };

  const h = (upperBound - lowerBound) / steps;
  let sum = f(lowerBound) + f(upperBound);

  for (let i = 1; i < steps; i++) {
    const x = lowerBound + i * h;
    sum += (i % 2 === 0 ? 2 : 4) * f(x);
  }

  return (h / 3) * sum;
}
