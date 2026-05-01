import { test } from 'node:test';
import assert from 'node:assert/strict';
import { evaluate } from '../src/math/evaluator.js';

test('evaluates simple arithmetic', () => {
  assert.equal(evaluate('2 + 3'), 5);
});

test('evaluates multiplication', () => {
  assert.equal(evaluate('4 * 7'), 28);
});

test('evaluates expression with variable', () => {
  assert.equal(evaluate('2 * x + 1', { x: 3 }), 7);
});

test('evaluates trigonometric functions', () => {
  const result = evaluate('sin(pi/2)');
  assert.ok(Math.abs(result - 1) < 1e-10);
});

test('evaluates sqrt', () => {
  assert.equal(evaluate('sqrt(16)'), 4);
});

test('evaluates logarithm', () => {
  const result = evaluate('log(e)');
  assert.ok(Math.abs(result - 1) < 1e-10);
});

test('throws on invalid expression', () => {
  assert.throws(() => evaluate('not_a_function(x)'), /Failed to evaluate/);
});

test('evaluates power expression', () => {
  assert.equal(evaluate('2^10'), 1024);
});
