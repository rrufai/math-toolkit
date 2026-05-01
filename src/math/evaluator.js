import { math, MAX_EXPRESSION_LENGTH } from './mathInstance.js';

/**
 * Evaluate a math expression with optional variable scope.
 * @param {string} expression - Math expression string
 * @param {Object} scope - Variable bindings, e.g. { x: 3 }
 * @returns {number|string} Result of the evaluation
 */
export function evaluate(expression, scope = {}) {
  if (expression.length > MAX_EXPRESSION_LENGTH) {
    throw new Error(`Expression is too long (max ${MAX_EXPRESSION_LENGTH} characters)`);
  }
  try {
    const result = math.evaluate(expression, scope);
    return result;
  } catch (err) {
    throw new Error(`Failed to evaluate "${expression}": ${err.message}`);
  }
}
