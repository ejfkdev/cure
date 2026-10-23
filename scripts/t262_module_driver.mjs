#!/usr/bin/env node
// Test262 **module 语义差分**驱动：每个用例作为 ESM 在独立 vm 上下文
// 链接（相对 fixture 由文件系统解析）并求值，比较状态 + console 输出。
// 需要 `node --experimental-vm-modules`。
import { createRequire } from 'module';
import vm from 'vm';
import path from 'path';
const require = createRequire(import.meta.url);
const fs = require('fs');

const input = fs.readFileSync(0, 'utf8');
const TIMEOUT = Number(process.env.T262_MODULE_TIMEOUT || 500);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const write = (s) => fs.writeSync(1, s + '\n');
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

async function runOne(job) {
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
    const cache = new Map();
    const load = (p) => {
      if (!cache.has(p)) {
        cache.set(p, new vm.SourceTextModule(fs.readFileSync(p, 'utf8'), {
          context: ctx, identifier: p,
        }));
      }
      return cache.get(p);
    };
    const rootId = path.join(job.dir, '__test__.mjs');
    const root = new vm.SourceTextModule(job.code, { context: ctx, identifier: rootId });
    await root.link(async (spec) => {
      const p = path.resolve(job.dir, spec);
      return load(p);
    });
    await Promise.race([root.evaluate(), sleep(TIMEOUT)]);
    if (job.async) {
      await Promise.race([doneP, sleep(TIMEOUT)]);
    }
  } catch (e) {
    status = 'throw:' + (e && e.constructor ? e.constructor.name : 'Unknown');
  }
  await sleep(0);
  await new Promise((r) => setImmediate(r));
  if (status === 'ok' && job.async && !done) status = 'async-timeout';
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
  const o = await runOne(job);
  const c = await runOne({ ...job, code: job.cured });
  write(JSON.stringify({
    id: job.id, st_orig: o.status, st_cured: c.status,
    out_orig: o.out, out_cured: c.out,
  }));
}
