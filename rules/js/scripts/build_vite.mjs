import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";

const srcDir = path.resolve(process.argv[2]);
const nodeExe = path.resolve(process.argv[3]);
const viteJs = path.resolve(process.argv[4]);
const outDir = process.argv[5];
const finalOut = path.resolve(process.argv[6]);

process.env.NODE_PATH = path.join(srcDir, 'node_modules');
process.chdir(srcDir);

const result = spawnSync(nodeExe, [viteJs, 'build'], {
    stdio: 'inherit',
    env: { ...process.env, NODE_ENV: 'production', NODE_PRESERVE_SYMLINKS: '1', NODE_OPTIONS: '--preserve-symlinks --preserve-symlinks-main' }
});

if (result.status !== 0) {
    process.exit(result.status !== null ? result.status : 1);
}

fs.renameSync(path.join(srcDir, outDir), finalOut);
