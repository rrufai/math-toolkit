import { test } from 'node:test';
import assert from 'node:assert/strict';
import { solveEquation, solveLinear } from '../src/math/solver.js';

test('solves x^2 - 4 = 0', () => {
  const roots = solveEquation('x^2 - 4');
  assert.equal(roots.length, 2);
  const sorted = roots.sort((a, b) => a - b);
  assert.ok(Math.abs(sorted[0] - (-2)) < 1e-5, `Expected -2, got ${sorted[0]}`);
  assert.ok(Math.abs(sorted[1] - 2) < 1e-5, `Expected 2, got ${sorted[1]}`);
});

test('solves x - 5 = 0', () => {
  const roots = solveEquation('x - 5');
  assert.equal(roots.length, 1);
  assert.ok(Math.abs(roots[0] - 5) < 1e-5);
});

test('solves x^3 - x = 0 (three roots)', () => {
  const roots = solveEquation('x^3 - x');
  const expectedRoots = [-1, 0, 1];
  assert.equal(roots.length, expectedRoots.length);
  assert.ok(
    expectedRoots.every((expected) =>
      roots.some((root) => Math.abs(root - expected) < 1e-5)
    ),
    `Expected roots to contain -1, 0, and 1, got ${JSON.stringify(roots)}`
  );
});

test('solves linear equation 2*x + 3 = 7', () => {
  const result = solveLinear('2*x + 3 = 7');
  assert.ok(Math.abs(result - 2) < 1e-5);
});

test('throws on missing = in solveLinear', () => {
  assert.throws(() => solveLinear('2*x + 3'), /exactly one "=" sign/);
});

test('throws on invalid expression', () => {
  assert.throws(() => solveEquation('unknownFunc(x)'), /Invalid expression/);
});
