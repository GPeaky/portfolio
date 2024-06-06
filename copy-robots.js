// Todo: Search an easy way to do this with vite

import fs from "fs";
import path from "path";

const __dirname = path.dirname(new URL(import.meta.url).pathname);

const src = path.resolve(__dirname, "public", "robots.txt");
const dest = path.resolve(__dirname, "dist", "robots.txt");

fs.copyFile(src, dest, (err) => {
  if (err) {
    console.error("Error copiando robots.txt:", err);
  } else {
    console.log("robots.txt copiado a dist/");
  }
});
