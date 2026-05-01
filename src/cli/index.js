import { Command } from 'commander';
import asciichart from 'asciichart';
import { evaluate } from '../math/evaluator.js';
import { solveEquation, solveLinear } from '../math/solver.js';
import { derivative, integrate } from '../math/calculus.js';

const program = new Command();

program
  .name('math-toolkit')
  .description('A command line math toolkit for solving equations, computing derivatives and integrals, and visualizing functions')
  .version('1.0.0');

program
  .command('eval <expression>')
  .description('Evaluate a math expression (e.g., "2 + 3 * sin(pi/4)")')
  .option('-v, --var <assignments>', 'Variable assignments as JSON (e.g. \'{"x":3}\')')
  .action((expression, options) => {
    try {
      const scope = options.var ? JSON.parse(options.var) : {};
      const result = evaluate(expression, scope);
      console.log(`\nExpression: ${expression}`);
      if (Object.keys(scope).length > 0) {
        console.log(`Variables:  ${JSON.stringify(scope)}`);
      }
      console.log(`Result:     ${result}\n`);
    } catch (err) {
      console.error(`\nError: ${err.message}\n`);
      process.exit(1);
    }
  });

program
  .command('solve <equation>')
  .description('Solve equation = 0 for x (e.g., "x^2 - 4")')
  .option('-v, --variable <name>', 'Variable name', 'x')
  .option('-g, --guess <number>', 'Initial guess', parseFloat, 1)
  .action((equation, options) => {
    try {
      const roots = solveEquation(equation, options.variable, options.guess);
      console.log(`\nEquation:   ${equation} = 0`);
      console.log(`Variable:   ${options.variable}`);
      if (roots.length === 0) {
        console.log('Roots:      No roots found in search range\n');
      } else {
        console.log(`Roots:      ${roots.join(', ')}\n`);
      }
    } catch (err) {
      console.error(`\nError: ${err.message}\n`);
      process.exit(1);
    }
  });

program
  .command('derivative <expression>')
  .description('Compute derivative of expression (e.g., "x^3 + 2*x")')
  .option('-v, --variable <name>', 'Variable name', 'x')
  .option('-p, --point <number>', 'Evaluate derivative at this point', parseFloat)
  .action((expression, options) => {
    try {
      const scope = options.point !== undefined ? { [options.variable]: options.point } : null;
      const result = derivative(expression, options.variable, scope);
      console.log(`\nExpression: ${expression}`);
      console.log(`d/d${options.variable}:     ${result.derivative}`);
      if (result.valueAtPoint !== undefined) {
        console.log(`At ${options.variable} = ${options.point}: ${result.valueAtPoint}`);
      }
      console.log();
    } catch (err) {
      console.error(`\nError: ${err.message}\n`);
      process.exit(1);
    }
  });

program
  .command('integral <expression> <lower> <upper>')
  .description('Compute definite integral (e.g., "x^2" 0 1)')
  .option('-v, --variable <name>', 'Variable name', 'x')
  .option('-n, --steps <number>', 'Number of steps for Simpson\'s rule', parseInt, 1000)
  .action((expression, lower, upper, options) => {
    try {
      const a = parseFloat(lower);
      const b = parseFloat(upper);
      const result = integrate(expression, options.variable, a, b, options.steps);
      console.log(`\nExpression: ${expression}`);
      console.log(`Bounds:     [${a}, ${b}]`);
      console.log(`∫ result:   ${result}\n`);
    } catch (err) {
      console.error(`\nError: ${err.message}\n`);
      process.exit(1);
    }
  });

program
  .command('plot <expression>')
  .description('Plot a function as ASCII chart (e.g., "sin(x)")')
  .option('--from <number>', 'Start of x range', parseFloat, -5)
  .option('--to <number>', 'End of x range', parseFloat, 5)
  .option('-n, --steps <number>', 'Number of plot points', parseInt, 60)
  .option('-v, --variable <name>', 'Variable name', 'x')
  .action((expression, options) => {
    try {
      const { from, to, steps, variable } = options;
      const series = [];
      const step = (to - from) / (steps - 1);
      for (let i = 0; i < steps; i++) {
        const xVal = from + i * step;
        const yVal = evaluate(expression, { [variable]: xVal });
        series.push(typeof yVal === 'number' ? yVal : NaN);
      }
      console.log(`\nf(${variable}) = ${expression}   [${from}, ${to}]\n`);
      console.log(asciichart.plot(series, { height: 15 }));
      console.log();
    } catch (err) {
      console.error(`\nError: ${err.message}\n`);
      process.exit(1);
    }
  });

export { program };
