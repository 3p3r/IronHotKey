/// <reference types="zx/globals" />

import fsp from "node:fs/promises";
import path from "node:path";
import { load } from "cheerio";

const ROOT = path.resolve(__dirname, "..");
const INDEX = path.join(ROOT, "reference", "docs", "lib", "index.htm");
const MODULES_DIR = path.join(ROOT, "crates", "ironhotkey-runtime", "src", "modules");

async function loadModuleFiles() {
  const entries = await fsp.readdir(MODULES_DIR, { withFileTypes: true });
  return entries
    .filter((entry) => entry.isFile() && entry.name.endsWith(".rs") && entry.name !== "mod.rs")
    .map((entry) => entry.name)
    .sort();
}

async function loadReferenceFunctions() {
  const html = await fsp.readFile(INDEX, "utf8");
  const $ = load(html);
  const names = new Set<string>();

  $("table.info td a").each((_, element) => {
    const text = $(element).text();
    for (const match of text.matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)\(\)/g)) {
      names.add(match[1]);
    }
  });

  return names;
}

async function loadRustMethods() {
  const moduleFiles = await loadModuleFiles();
  const names = new Set<string>();
  for (const file of moduleFiles) {
    const src = await fsp.readFile(path.join(MODULES_DIR, file), "utf8");
    for (const match of src.matchAll(/\("([A-Za-z_][A-Za-z0-9_]*)",\s*[A-Za-z_][A-Za-z0-9_]*\)/g)) {
      names.add(match[1]);
    }
  }
  return names;
}

async function main() {
  const [reference, rust] = await Promise.all([loadReferenceFunctions(), loadRustMethods()]);

  const missing = [...reference].filter((name) => !rust.has(name)).sort();

  if (missing.length === 0) {
    console.log(
      chalk.green(`✅ 100% coverage: ${reference.size}/${reference.size} reference functions found in Rust METHODS.`),
    );
    return;
  }

  console.error(chalk.red(`❌ Coverage gap: ${missing.length} missing of ${reference.size} reference functions.`));
  for (const name of missing) console.error(` - ${name}`);
  process.exit(1);
}

main().catch((error) => {
  console.error(chalk.red(error));
  process.exit(1);
});
