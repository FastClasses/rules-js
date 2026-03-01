import { spawnSync } from "node:child_process";
import path from "node:path";
import process from "node:process";

const exe = path.resolve(process.argv[2]);
const srcDir = path.resolve(process.argv[3]);
const entry = process.argv[4];

process.env.NODE_PATH = path.join(srcDir, 'node_modules');

const isDeno = exe.endsWith('deno') || exe.endsWith('deno.exe');
const runArgs = isDeno ? ["run", "-A", entry] : [entry];

const result = spawnSync(exe, runArgs, {
    stdio: 'inherit',
    env: process.env,
    cwd: srcDir,
});

process.exit(result.status !== null ? result.status : 1);
