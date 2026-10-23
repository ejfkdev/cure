#!/usr/bin/env node
// Test262 **编译差分**驱动：只编译不执行。原版与 cure 输出分别在全新 vm
// 上下文编译（script 用 vm.Script；module 用 vm.SourceTextModule），
// 返回 ok / syntax（SyntaxError）/ other。
// 需要 `node --experimental-vm-modules`（module 用例）。
import { createRequire } from 'module';
import vm from 'vm';
const require = createRequire(import.meta.url);
const fs = require('fs');

const input = fs.readFileSync(0, 'utf8');
const out = [];
function compileOne(code, isModule) {
  try {
    const ctx = vm.createContext({});
    if (isModule) {
      new vm.SourceTextModule(code, { context: ctx });
    } else {
      new vm.Script(code, { filename: 'test.js' });
    }
    return 'ok';
  } catch (e) {
    const name = e && e.constructor ? e.constructor.name : 'Unknown';
    return name === 'SyntaxError' ? 'syntax' : 'other:' + name;
  }
}
for (const line of input.split('\n')) {
  if (!line.trim()) continue;
  const j = JSON.parse(line);
  out.push(JSON.stringify({
    id: j.id,
    orig: compileOne(j.orig, j.module === true),
    cured: compileOne(j.cured, j.module === true),
  }));
}
fs.writeSync(1, out.join('\n') + '\n');
