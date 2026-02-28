const path = require('path');
const { spawnSync } = require('child_process');

const nodeExe = path.resolve(process.argv[2]);
const srcDir = path.resolve(process.argv[3]);
const eslintJs = path.join(srcDir, 'node_modules', 'eslint', 'bin', 'eslint.js');

process.env.NODE_PATH = path.join(srcDir, 'node_modules');
process.chdir(srcDir);

const result = spawnSync(nodeExe, [eslintJs, '.'], {
    stdio: 'inherit',
    env: process.env
});

process.exit(result.status !== null ? result.status : 1);
