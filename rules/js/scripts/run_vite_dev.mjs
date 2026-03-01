import path from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";

const nodeExe = path.resolve(process.argv[2]);
const srcDir = path.resolve(process.argv[3]);
const viteJs = path.join(srcDir, 'node_modules', 'vite', 'bin', 'vite.js');

process.env.NODE_PATH = path.join(srcDir, 'node_modules');

const result = spawnSync(nodeExe, [viteJs, 'dev', '--host'], {
    stdio: 'inherit',
    env: process.env,
    cwd: srcDir,
});

process.exit(result.status !== null ? result.status : 1);
