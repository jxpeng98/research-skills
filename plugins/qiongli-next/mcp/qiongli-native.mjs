#!/usr/bin/env node
import { spawn } from 'node:child_process';
const args = ['--yes', 'qiongli@2.0.0-alpha.8', 'mcp', 'serve', '--profile', 'lite', '--transport', 'stdio'];
const windows = process.platform === 'win32';
const command = windows ? ['npx.cmd', ...args].join(' ') : 'npx';
const child = spawn(command, windows ? [] : args, { stdio: 'inherit', shell: windows });
for (const signal of ['SIGINT', 'SIGTERM', 'SIGHUP']) process.on(signal, () => child.kill(signal));
child.on('error', () => { console.error('Unable to start the pinned Qiongli native MCP.'); process.exitCode = 1; });
child.on('close', (code, signal) => {
  if (signal) { process.removeAllListeners(signal); process.kill(process.pid, signal); }
  else process.exitCode = code ?? 1;
});
