import path from "node:path";
import fs from "node:fs";
import { spawnSync } from "node:child_process";
import process from "node:process";

const nodeExe = path.resolve(process.argv[2]);
const srcDir = path.resolve(process.argv[3]);
const stampFile = path.resolve(process.argv[4]);
const tscBin = path.join(srcDir, 'node_modules', '.bin', 'tsc');
const tscJs = path.join(srcDir, 'node_modules', 'typescript', 'lib', 'tsc.js');

process.env.NODE_PATH = path.join(srcDir, 'node_modules');
process.chdir(srcDir);

const tsc = fs.existsSync(tscJs) ? tscJs : tscBin;

const result = spawnSync(nodeExe, [tsc, '--noEmit'], {
    stdio: 'inherit',
    env: process.env
});

if (result.status !== 0) {
    process.exit(result.status !== null ? result.status : 1);
}

fs.writeFileSync(stampFile, 'OK');
