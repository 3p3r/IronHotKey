/// <reference types="zx/globals" />

import fs from "node:fs";
import fsp from "node:fs/promises";
import path from "node:path";
import os from "node:os";
import download from "download";

const REPO_TARBALL = "https://codeload.github.com/alfredomtx/tree-sitter-autohotkey/tar.gz/refs/heads/master";
const ROOT = path.resolve(__dirname, "..");
const VENDOR = path.join(ROOT, "crates", "ironhotkey-grammar", "vendor");

const REQUIRED_FILES = ["src/parser.c", "src/tree_sitter/parser.h"];

async function main() {
  console.log(chalk.blue("Fetching tree-sitter-autohotkey grammar..."));

  if (fs.existsSync(VENDOR)) {
    console.log(chalk.yellow(`Already exists: ${VENDOR}`));
    return;
  }

  await fsp.mkdir(VENDOR, { recursive: true });
  const tmpDir = await fsp.mkdtemp(path.join(os.tmpdir(), "ironhotkey-grammar-"));
  const extractDir = path.join(tmpDir, "extract");

  console.log(chalk.blue(`Downloading and extracting grammar from ${REPO_TARBALL}`));
  await download(REPO_TARBALL, extractDir, { extract: true, strip: 1 });

  for (const rel of REQUIRED_FILES) {
    const src = path.join(extractDir, rel);
    const dst = path.join(VENDOR, rel);
    await fsp.mkdir(path.dirname(dst), { recursive: true });
    await fsp.copyFile(src, dst);
  }

  const scannerSrc = path.join(extractDir, "src/scanner.c");
  const scannerDst = path.join(VENDOR, "src/scanner.c");
  if (fs.existsSync(scannerSrc)) {
    await fsp.copyFile(scannerSrc, scannerDst);
  }

  console.log(chalk.green("Grammar fetched successfully!"));
}

main().catch((error) => {
  console.error(chalk.red(error));
  process.exit(1);
});
