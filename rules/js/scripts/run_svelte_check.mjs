import path from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";

const nodeExe = path.resolve(process.argv[2]);
const srcDir = path.resolve(process.argv[3]);
const svelteKitJs = path.join(srcDir, 'node_modules', '@sveltejs', 'kit', 'svelte-kit.js');
const svelteCheckJs = path.join(srcDir, 'node_modules', 'svelte-check', 'bin', 'svelte-check');

process.env.NODE_PATH = path.join(srcDir, 'node_modules');
process.chdir(srcDir);

const syncResult = spawnSync(nodeExe, [svelteKitJs, 'sync'], {
    stdio: 'inherit',
    env: process.env
});

if (syncResult.status !== 0) {
    process.exit(syncResult.status !== null ? syncResult.status : 1);
}

const checkResult = spawnSync(nodeExe, [svelteCheckJs, '--tsconfig', './tsconfig.json'], {
    stdio: 'inherit',
    env: process.env
});

process.exit(checkResult.status !== null ? checkResult.status : 1);
