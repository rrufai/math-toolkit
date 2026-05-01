import { create, all } from 'mathjs';

const math = create(all);

/**
 * Evaluate a math expression with optional variable scope.
 * @param {string} expression - Math expression string
 * @param {Object} scope - Variable bindings, e.g. { x: 3 }
 * @returns {number|string} Result of the evaluation
 */
export function evaluate(expression, scope = {}) {
  try {
    const result = math.evaluate(expression, scope);
    return result;
  } catch (err) {
    throw new Error(`Failed to evaluate "${expression}": ${err.message}`);
  }
}
