import { test } from 'node:test';
import assert from 'node:assert/strict';
import { derivative, integrate } from '../src/math/calculus.js';
import { evaluate } from '../src/math/evaluator.js';

test('derivative of x^2 is 2*x', () => {
  const result = derivative('x^2');
  // mathjs returns "2 * x" or "x * 2"
  assert.ok(result.derivative.includes('2') && result.derivative.includes('x'));
});

test('derivative of x^3 at x=2 is 12', () => {
  const result = derivative('x^3', 'x', { x: 2 });
  assert.ok(Math.abs(result.valueAtPoint - 12) < 1e-6);
});

test('derivative of constant is 0', () => {
  const result = derivative('5', 'x');
  assert.equal(result.derivative, '0');
});

test('derivative of sin(x) is cos(x)', () => {
  const result = derivative('sin(x)', 'x');
  assert.ok(result.derivative.toLowerCase().includes('cos'));
});

test('integrates x^2 from 0 to 1 (result ~0.333)', () => {
  const result = integrate('x^2', 'x', 0, 1);
  assert.ok(Math.abs(result - 1/3) < 1e-6);
});

test('integrates sin(x) from 0 to pi (result ~2)', () => {
  const pi = evaluate('pi');
  const result = integrate('sin(x)', 'x', 0, pi);
  assert.ok(Math.abs(result - 2) < 1e-4);
});

test('integrates constant 1 from 0 to 5 (result = 5)', () => {
  const result = integrate('1', 'x', 0, 5);
  assert.ok(Math.abs(result - 5) < 1e-6);
});

test('throws on non-numeric bounds', () => {
  assert.throws(() => integrate('x', 'x', 'a', 1), /must be finite numbers/);
});
