#!/usr/bin/env node
// 通用**编译差分**驱动（流式）：stdin 每行 {id, orig, cured, module}，
// 只编译不执行，逐条输出 {id, orig, cured}。
// 需要 `node --experimental-vm-modules`（module 用例）。
import { createRequire } from 'module';
import vm from 'vm';
import readline from 'readline';
const require = createRequire(import.meta.url);
const fs = require('fs');
const write = (s) => fs.writeSync(1, s + '\n');

function compileOne(code, isModule) {
  try {
    const ctx = vm.createContext({});
    if (isModule) new vm.SourceTextModule(code, { context: ctx });
    else new vm.Script(code, { filename: 'test.js' });
    return 'ok';
  } catch (e) {
    const name = e && e.constructor ? e.constructor.name : 'Unknown';
    return name === 'SyntaxError' ? 'syntax' : 'other:' + name;
  }
}

const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
for await (const line of rl) {
  if (!line.trim()) continue;
  const j = JSON.parse(line);
  write(JSON.stringify({
    id: j.id,
    orig: compileOne(j.orig, j.module === true),
    cured: compileOne(j.cured, j.module === true),
  }));
}
