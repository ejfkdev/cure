#!/usr/bin/env node
// Test262 语义差分驱动（同步版）：stdin 全量读 JSONL，逐用例在全新 vm
// 上下文执行，stdout 一次性输出 JSONL 结果。
import { createRequire } from 'module';
import vm from 'vm';
const require = createRequire(import.meta.url);
const fs = require('fs');

// 部分用例故意制造未处理 rejection（异步/Promise 语义测试）——驱动
// 只关心同步执行结果，忽略它们，避免 node 以 ERR_UNHANDLED_REJECTION 退出。
process.on('unhandledRejection', () => {});

const input = fs.readFileSync(0, 'utf8');
const lines = input.split('\n');
const outChunks = [];

function fmt(v) {
  try {
    if (v === undefined) return 'undefined';
    if (v === null) return 'null';
    if (typeof v === 'object') return JSON.stringify(v);
    return String(v);
  } catch {
    return '<unfmt>';
  }
}

function runOne(code) {
  const out = [];
  const sandbox = {
    console: {
      log: (...a) => out.push(a.map(fmt).join(' ')),
      error: (...a) => out.push(a.map(fmt).join(' ')),
      warn: (...a) => out.push(a.map(fmt).join(' ')),
      info: (...a) => out.push(a.map(fmt).join(' ')),
      debug: (...a) => out.push(a.map(fmt).join(' ')),
    },
    print: (...a) => out.push(a.map(fmt).join(' ')),
    $DONE: () => {},
    setTimeout: (fn) => { try { fn(); } catch (e) { throw e; } return 0; },
    clearTimeout: () => {},
    queueMicrotask: (fn) => { try { fn(); } catch (e) { throw e; } },
  };
  sandbox.globalThis = sandbox;
  let status = 'ok';
  try {
    const ctx = vm.createContext(sandbox);
    vm.runInContext(code, ctx, { timeout: 500, filename: 'test.js' });
  } catch (e) {
    status = 'throw:' + (e && e.constructor ? e.constructor.name : 'Unknown');
  }
  return { status, out: out.join('\n') };
}

for (const line of lines) {
  if (!line.trim()) continue;
  const job = JSON.parse(line);
  const o = runOne(job.orig);
  const c = runOne(job.cured);
  outChunks.push(
    JSON.stringify({ id: job.id, st_orig: o.status, st_cured: c.status, out_orig: o.out, out_cured: c.out })
  );
}
fs.writeSync(1, outChunks.join('\n') + '\n');
