const path = require('path');
const { spawnSync } = require('child_process');

const nodeExe = path.resolve(process.argv[2]);
const srcDir = path.resolve(process.argv[3]);
const entry = process.argv[4];

process.env.NODE_PATH = path.join(srcDir, 'node_modules');
process.chdir(srcDir);

const result = spawnSync(nodeExe, [entry], {
    stdio: 'inherit',
    env: process.env
});

process.exit(result.status !== null ? result.status : 1);
