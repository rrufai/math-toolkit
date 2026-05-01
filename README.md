# Math Toolkit

A web and command line set of mathematical tools for solving equations, computing derivatives and integrals, and visualizing functions.

## Features

- **Expression Evaluator** – Evaluate math expressions with variable support
- **Equation Solver** – Find roots using Newton's method
- **Derivative Calculator** – Symbolic differentiation via mathjs
- **Integral Calculator** – Numerical integration via Simpson's rule
- **Function Plotter** – ASCII plots (CLI) and interactive Chart.js plots (Web)

## Installation

```bash
npm install
```

## Usage

### Web Server

```bash
npm start
# Open http://localhost:3000
```

### CLI

```bash
# Evaluate an expression
npx math-toolkit eval "2 + 3 * sin(pi/4)"

# Solve equation = 0
npx math-toolkit solve "x^2 - 4"

# Compute derivative
npx math-toolkit derivative "x^3 + 2*x"

# Compute definite integral
npx math-toolkit integral "x^2" 0 1

# Plot a function
npx math-toolkit plot "sin(x)" --from -6.28 --to 6.28
```

## Tests

```bash
npm test
```

## API Endpoints

| Method | Endpoint | Body | Response |
|--------|----------|------|----------|
| POST | /api/evaluate | `{ expression, scope? }` | `{ result }` |
| POST | /api/solve | `{ equation, variable? }` | `{ roots }` |
| POST | /api/derivative | `{ expression, variable?, point? }` | `{ derivative, valueAtPoint? }` |
| POST | /api/integrate | `{ expression, variable?, lower, upper }` | `{ result }` |
| POST | /api/plot | `{ expression, from?, to?, steps? }` | `{ points }` |

## License

MIT