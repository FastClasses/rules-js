import path from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";

const nodeExe = path.resolve(process.argv[2]);
const srcDir = path.resolve(process.argv[3]);
const vitestJs = path.join(srcDir, 'node_modules', 'vitest', 'vitest.mjs');

process.env.NODE_PATH = path.join(srcDir, 'node_modules');

process.chdir(srcDir);

const result = spawnSync(nodeExe, [vitestJs, 'run', '--passWithNoTests'], {
    stdio: 'inherit',
    env: process.env
});

process.exit(result.status !== null ? result.status : 1);
