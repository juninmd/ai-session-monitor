#!/usr/bin/env node
// Boot benchmark for the flyout's web bundle: how long a built `dist` takes from navigation to the
// first settled frame, how much script it runs to get there and how much it ships. Compares any
// number of builds side by side, so a change can be measured against the commit before it.
//
//   node scripts/bench-boot.mjs before=/path/to/old/dist after=dist [--runs 15] [--cpu 4] [--locale pt-BR]
//
// Needs Playwright with Chromium on the machine (it is not a dependency of this repository):
// `npx playwright install chromium`, or NODE_PATH pointing at a global install. The page runs on
// the mock backend (`?mock=demo`), so nothing is read from the profile and no network is used.
import { readdir, readFile, stat } from 'node:fs/promises';
import { createServer } from 'node:http';
import { createRequire } from 'node:module';
import { extname, join, normalize, resolve } from 'node:path';
import { gzipSync } from 'node:zlib';

const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' };
const READY = '.nx-fly[data-loaded]';

function parseArgs(argv) {
  const options = { builds: [], runs: 15, cpu: 4, locale: 'en' };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--runs') options.runs = Number(argv[++index]);
    else if (arg === '--cpu') options.cpu = Number(argv[++index]);
    else if (arg === '--locale') options.locale = argv[++index];
    else if (arg.includes('=')) {
      const [label, dir] = arg.split('=');
      options.builds.push({ label, dir: resolve(dir) });
    } else options.builds.push({ label: arg, dir: resolve(arg) });
  }
  if (options.builds.length === 0) options.builds.push({ label: 'dist', dir: resolve('dist') });
  return options;
}

function serve(root) {
  const server = createServer(async (request, response) => {
    const path = normalize(new URL(request.url, 'http://x').pathname);
    const file = join(root, path === '/' ? 'index.html' : path);
    try {
      if (!file.startsWith(root)) throw new Error('outside the build');
      const body = await readFile(file);
      response.writeHead(200, {
        'content-type': TYPES[extname(file)] ?? 'application/octet-stream',
      });
      response.end(body);
    } catch {
      response.writeHead(404).end();
    }
  });
  return new Promise((done) => {
    server.listen(0, '127.0.0.1', () => done({ server, port: server.address().port }));
  });
}

const median = (values) => {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
};

async function sizes(dir) {
  const assets = join(dir, 'assets');
  const files = (await readdir(assets)).filter((name) => name.endsWith('.js'));
  const entry = files.find((name) => name.startsWith('index-'));
  const body = entry ? await readFile(join(assets, entry)) : Buffer.alloc(0);
  let total = 0;
  for (const name of files) total += (await stat(join(assets, name))).size;
  return {
    entryKb: body.length / 1024,
    entryGzipKb: gzipSync(body).length / 1024,
    allKb: total / 1024,
  };
}

async function measure(chromium, url, { runs, cpu, locale }) {
  const browser = await chromium.launch();
  const samples = { ready: [], script: [], heap: [], shipped: [] };
  // One warm-up run is thrown away: the first launch pays for the browser, not for the bundle.
  for (let run = -1; run < runs; run += 1) {
    const context = await browser.newContext({ viewport: { width: 432, height: 640 } });
    const page = await context.newPage();
    await page.addInitScript(
      ({ selector, language }) => {
        localStorage.setItem('aism.locale', language);
        const seen = () => {
          if (window.__ready === undefined && document.querySelector(selector)) {
            window.__ready = performance.now();
          }
        };
        new MutationObserver(seen).observe(document, {
          subtree: true,
          childList: true,
          attributes: true,
        });
      },
      { selector: READY, language: locale },
    );
    const session = await context.newCDPSession(page);
    await session.send('Performance.enable');
    await session.send('Emulation.setCPUThrottlingRate', { rate: cpu });
    let shipped = 0;
    page.on('response', async (response) => {
      if (!response.url().endsWith('.js')) return;
      shipped += (await response.body().catch(() => Buffer.alloc(0))).length;
    });
    await page.goto(url);
    await page.waitForFunction(() => window.__ready !== undefined);
    const ready = await page.evaluate(() => window.__ready);
    const { metrics } = await session.send('Performance.getMetrics');
    const metric = (name) => metrics.find((item) => item.name === name)?.value ?? 0;
    if (run >= 0) {
      samples.ready.push(ready);
      samples.script.push(metric('ScriptDuration') * 1000);
      samples.heap.push(metric('JSHeapUsedSize') / 1024 / 1024);
      samples.shipped.push(shipped / 1024);
    }
    await context.close();
  }
  await browser.close();
  return Object.fromEntries(Object.entries(samples).map(([key, values]) => [key, median(values)]));
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  let chromium;
  try {
    ({ chromium } = createRequire(import.meta.url)('playwright'));
  } catch {
    console.error('Playwright is not installed here; see the comment at the top of this script.');
    process.exit(2);
  }
  console.log(
    `runs: ${options.runs} (median) · CPU throttle: ${options.cpu}x · language: ${options.locale}\n`,
  );
  console.log(
    '| build | entry JS | entry gzip | all JS on disk | JS loaded | ready | script time | JS heap |',
  );
  console.log('|---|--:|--:|--:|--:|--:|--:|--:|');
  for (const build of options.builds) {
    const size = await sizes(build.dir);
    const { server, port } = await serve(build.dir);
    const result = await measure(chromium, `http://127.0.0.1:${port}/?mock=demo`, options);
    server.close();
    const kb = (value) => `${value.toFixed(1)} kB`;
    const ms = (value) => `${value.toFixed(0)} ms`;
    console.log(
      `| ${build.label} | ${kb(size.entryKb)} | ${kb(size.entryGzipKb)} | ${kb(size.allKb)} | ${kb(result.shipped)} | ${ms(result.ready)} | ${ms(result.script)} | ${result.heap.toFixed(1)} MB |`,
    );
  }
}

await main();
