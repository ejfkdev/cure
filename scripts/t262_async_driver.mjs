#!/usr/bin/env node
// Test262 **async 语义差分**驱动：逐用例在全新 vm 上下文执行，等待
// $DONE / 微任务 / 定时器排空后返回 状态 + console 输出。
// 每个用例同步超时 500ms，async 等待至多 $T262_ASYNC_TIMEOUT（默认 500ms）。
import { createRequire } from 'module';
import vm from 'vm';
const require = createRequire(import.meta.url);
const fs = require('fs');

const input = fs.readFileSync(0, 'utf8');
const TIMEOUT = Number(process.env.T262_ASYNC_TIMEOUT || 500);
let rejections = [];
process.on('unhandledRejection', (r) => { rejections.push(r); });

function fmt(v) {
  try {
    if (v === undefined) return 'undefined';
    if (v === null) return 'null';
    if (typeof v === 'object') return JSON.stringify(v);
    return String(v);
  } catch { return '<unfmt>'; }
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const write = (s) => fs.writeSync(1, s + '\n');

async function runOne(code) {
  rejections = [];
  const out = [];
  let done = false;
  let doneResolve;
  const doneP = new Promise((r) => { doneResolve = r; });
  const timers = new Set();
  const sandbox = {
    console: {
      log: (...a) => out.push(a.map(fmt).join(' ')),
      error: (...a) => out.push(a.map(fmt).join(' ')),
      warn: (...a) => out.push(a.map(fmt).join(' ')),
      info: (...a) => out.push(a.map(fmt).join(' ')),
      debug: (...a) => out.push(a.map(fmt).join(' ')),
    },
    print: (...a) => out.push(a.map(fmt).join(' ')),
    $DONE: () => { done = true; doneResolve(); },
    setTimeout: (fn, ms) => {
      const h = setTimeout(() => {
        timers.delete(h);
        try { fn(); } catch (e) { rejections.push(e); }
      }, Math.min(Number(ms) || 0, 400));
      timers.add(h);
      return h;
    },
    clearTimeout: (h) => { timers.delete(h); clearTimeout(h); },
    setInterval: () => 0,
    clearInterval: () => {},
    queueMicrotask: (fn) => { try { fn(); } catch (e) { rejections.push(e); } },
  };
  sandbox.globalThis = sandbox;
  let status = 'ok';
  try {
    const ctx = vm.createContext(sandbox);
    vm.runInContext(code, ctx, { timeout: 500, filename: 'test.js' });
  } catch (e) {
    status = 'throw:' + (e && e.constructor ? e.constructor.name : 'Unknown');
  }
  if (status === 'ok') {
    await Promise.race([doneP, sleep(TIMEOUT)]);
    await sleep(0);
    await new Promise((r) => setImmediate(r));
  }
  if (status === 'ok' && !done) status = 'async-timeout';
  if (status === 'ok' && rejections.length > 0) {
    const e = rejections[0];
    status = 'throw:' + (e && e.constructor ? e.constructor.name : 'Unknown');
  }
  for (const h of timers) clearTimeout(h);
  timers.clear();
  return { status, out: out.join('\n') };
}

for (const line of input.split('\n')) {
  if (!line.trim()) continue;
  const job = JSON.parse(line);
  const o = await runOne(job.orig);
  const c = await runOne(job.cured);
  // 逐条写出：个别用例可能令 V8 崩溃，已完成结果不丢（分块排查）
  write(JSON.stringify({
    id: job.id, st_orig: o.status, st_cured: c.status,
    out_orig: o.out, out_cured: c.out,
  }));
}
