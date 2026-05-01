import express from 'express';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { apiRouter } from './routes/api.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const app = express();
const PORT = process.env.PORT || 3000;

app.use(express.json());
app.use(express.static(join(__dirname, 'public')));
app.use('/api', apiRouter);

// Only bind the port when this file is executed directly (not imported).
if (process.argv[1] === __filename) {
  app.listen(PORT, () => {
    console.log(`Math Toolkit server running at http://localhost:${PORT}`);
  });
}

export { app };
