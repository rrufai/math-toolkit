import { Router } from 'express';
import { evaluate } from '../../math/evaluator.js';
import { solveEquation } from '../../math/solver.js';
import { derivative, integrate } from '../../math/calculus.js';

const apiRouter = Router();

apiRouter.post('/evaluate', (req, res) => {
  try {
    const { expression, scope = {} } = req.body;
    if (!expression) return res.status(400).json({ error: 'expression is required' });
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
    const result = integrate(expression, variable, parseFloat(lower), parseFloat(upper));
    res.json({ result });
  } catch (err) {
    res.status(400).json({ error: err.message });
  }
});

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

    const points = [];
    const step = (plotTo - plotFrom) / (plotSteps - 1);
    for (let i = 0; i < plotSteps; i++) {
      const x = plotFrom + i * step;
      try {
        const y = evaluate(expression, { [variable]: x });
        points.push({ x, y: typeof y === 'number' ? y : null });
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
