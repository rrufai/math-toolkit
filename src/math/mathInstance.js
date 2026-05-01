import { create, all } from 'mathjs';

/**
 * Shared mathjs instance used across all math modules.
 *
 * Using a single instance avoids duplicating the `create(all)` setup and
 * lets us consistently apply the same restrictions in one place.
 *
 * Restrictions applied:
 *   - `import`      — not needed; could allow callers to inject arbitrary JS.
 *   - `createUnit`  — unit creation is not part of this toolkit's feature set.
 */
const math = create(all);

math.import(
  {
    import: () => {
      throw new Error('Function import is disabled');
    },
    createUnit: () => {
      throw new Error('Function createUnit is disabled');
    },
  },
  { override: true }
);

/** Maximum allowed expression length (characters). */
export const MAX_EXPRESSION_LENGTH = 2000;

export { math };
