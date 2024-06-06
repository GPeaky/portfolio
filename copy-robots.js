// Todo: Search an easy way to do this with vite

import fs from "fs/promises";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const src = path.resolve(__dirname, "public", "robots.txt");
const dest = path.resolve(__dirname, "dist", "robots.txt");

fs.copyFile(src, dest)
  .then(() => console.log("robots.txt copiado a dist/"))
  .catch((err) => console.error("Error copiando robots.txt:", err));
