#!/usr/bin/env node
/**
 * check-i18n.js — zero-drift parity guard for the bilingual docs site.
 *
 * 1. Walks docs/ for canonical (English) Markdown pages, excluding
 *    .vitepress, locale folders, public assets and the legacy notebook
 *    pages listed in `srcExclude` (docs/.vitepress/shared.ts — single
 *    source of truth, parsed at runtime).
 * 2. Compares against each locale folder (docs/pt/, docs/es/, …).
 * 3. Reports missing mirrors AND orphan mirrors (1:1 symmetric tree).
 * 4. Optionally validates the full build (`--build`), which enforces
 *    zero broken links in every locale (ignoreDeadLinks: false).
 *
 * Usage:
 *   node scripts/check-i18n.js            # parity report only
 *   node scripts/check-i18n.js --build    # parity + `npm run build` in docs/
 */
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname } from 'node:path';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const DOCS = join(ROOT, 'docs');
const SHARED_TS = join(DOCS, '.vitepress', 'shared.ts');

const IGNORED_DIRS = new Set(['.vitepress', '.git', 'node_modules', 'public', '.vitepress-dist', 'dist', 'cache']);
const LOCALE_DIR_RE = /^[a-z]{2}(-[A-Z]{2})?$/; // pt, es, fr, pt-BR, …

/** Read `srcExclude: [...]` globs from shared.ts (single source of truth). */
function readSrcExclude() {
  try {
    const src = readFileSync(SHARED_TS, 'utf8');
    const key = src.indexOf('srcExclude');
    if (key === -1) return [];
    const open = src.indexOf('[', key);
    if (open === -1) return [];
    // Scan with bracket-depth counting, skipping brackets inside quotes
    // (globs like '[0-9]*.md' contain ']' that must not close the array).
    let depth = 0;
    let quote = null;
    let end = -1;
    for (let i = open; i < src.length; i++) {
      const c = src[i];
      if (quote) {
        if (c === quote && src[i - 1] !== '\\') quote = null;
        continue;
      }
      if (c === "'" || c === '"' || c === '`') quote = c;
      else if (c === '[') depth++;
      else if (c === ']') {
        depth--;
        if (depth === 0) {
          end = i;
          break;
        }
      }
    }
    if (end === -1) return [];
    const body = src.slice(open, end + 1);
    return [...body.matchAll(/['"]([^'"]+)['"]/g)].map((g) => g[1]);
  } catch {
    return [];
  }
}

/** Minimal glob → RegExp (supports *, ?, [...] — enough for srcExclude). */
function globToRegExp(glob) {
  let re = '';
  for (let i = 0; i < glob.length; i++) {
    const c = glob[i];
    if (c === '*') re += '[^/]*';
    else if (c === '?') re += '[^/]';
    else if (c === '[') {
      const j = glob.indexOf(']', i);
      re += j === -1 ? '\\[' : glob.slice(i, j + 1);
      if (j !== -1) i = j;
    } else if ('\\.+^${}()|'.includes(c)) re += `\\${c}`;
    else re += c;
  }
  return new RegExp(`^${re}$`);
}

function walk(dir, base = dir) {
  const out = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    const st = statSync(full);
    if (st.isDirectory()) {
      if (dir === base && (IGNORED_DIRS.has(entry) || LOCALE_DIR_RE.test(entry))) continue;
      if (IGNORED_DIRS.has(entry)) continue;
      out.push(...walk(full, base));
    } else if (entry.endsWith('.md')) {
      out.push(relative(base, full).split(sep).join('/'));
    }
  }
  return out.sort();
}

function main() {
  const srcExclude = readSrcExclude().map(globToRegExp);
  const isExcluded = (rel) => srcExclude.some((re) => re.test(rel));

  const allRoot = walk(DOCS);
  const canonical = allRoot.filter((f) => {
    const top = f.split('/')[0];
    if (LOCALE_DIR_RE.test(top)) return false; // locale mirror, not canonical
    if (isExcluded(f)) return false; // legacy notebook page
    return true;
  });

  // Locale folders = two-letter dirs containing an index.md (pt, es, …).
  const locales = readdirSync(DOCS).filter((e) => {
    if (!LOCALE_DIR_RE.test(e)) return false;
    try {
      return statSync(join(DOCS, e)).isDirectory();
    } catch {
      return false;
    }
  });

  if (locales.length === 0) {
    console.error('check-i18n: no locale folders found under docs/ (expected docs/pt/, …)');
    process.exit(2);
  }

  console.log(`check-i18n: ${canonical.length} canonical page(s), locale(s): ${locales.join(', ')}`);
  let failed = false;

  for (const locale of locales) {
    const mirrored = new Set(walk(join(DOCS, locale)));
    const missing = canonical.filter((f) => !mirrored.has(f));
    const orphan = [...mirrored].filter((f) => !canonical.includes(f) && !isExcluded(f));

    if (missing.length === 0 && orphan.length === 0) {
      console.log(`  [${locale}] OK — ${canonical.length}/${canonical.length} mirrored, zero drift`);
      continue;
    }
    failed = true;
    for (const f of missing) console.error(`  [${locale}] MISSING  docs/${locale}/${f}  (no mirror of docs/${f})`);
    for (const f of orphan) console.error(`  [${locale}] ORPHAN   docs/${locale}/${f}  (no canonical docs/${f})`);
  }

  if (failed) {
    console.error('\ncheck-i18n: FAILED — restore the 1:1 symmetric tree (see contributing/translations).');
    process.exit(1);
  }
  console.log('check-i18n: PASSED — zero drift.\n');

  if (process.argv.includes('--build')) {
    console.log('check-i18n: running docs build (zero broken links in all locales)…');
    const pm = (() => {
      try {
        const r = spawnSync('pnpm', ['--version'], { stdio: 'ignore' });
        return r.status === 0 ? 'pnpm' : 'npm';
      } catch {
        return 'npm';
      }
    })();
    const build = spawnSync(pm, ['run', 'build'], { cwd: DOCS, stdio: 'inherit' });
    if (build.status !== 0) {
      console.error('check-i18n: BUILD FAILED — fix dead links (ignoreDeadLinks: false).');
      process.exit(build.status ?? 1);
    }
    console.log('check-i18n: BUILD PASSED.');
  }
}

main();
