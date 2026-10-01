#!/usr/bin/env node
// Generates the source reference used by ADR-002. No test result is inferred.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const root = path.resolve(__dirname, '..');
const read = (name) => fs.readFileSync(path.join(root, name), 'utf8');
const files = [
  'crates/petunia_design_foundation/src/lib.rs',
  'crates/petunia_design_document/src/document_object.rs',
  'crates/petunia_design_document/src/document.rs',
  'crates/petunia_design_document/src/changeset.rs',
  'crates/petunia_design_document/src/validation.rs',
  'crates/petunia_design_document/src/modifiers.rs',
  'crates/petunia_design_document/src/mutator.rs',
  'crates/petunia_design_application/src/transaction.rs',
  'crates/petunia_design_application/src/history.rs',
  'crates/petunia_design_application/src/commands.rs',
  'crates/petunia_design_io/src/package.rs',
  'crates/petunia_design_raster/src/tile.rs',
  'crates/petunia_design_raster/src/brush.rs',
  'crates/petunia_design_testkit/src/bin/canvas_benchmark.rs',
  'crates/petunia_design_shell/src/tools/perspective.rs',
  'crates/petunia_design_shell/src/tools/gradient.rs',
  'crates/petunia_design_shell/src/tools/photo.rs',
  'apps/petunia-design/src/main.rs',
];
const tests = [
  'crates/petunia_design_application/tests/correctness_regressions.rs',
  'crates/petunia_design_application/tests/transaction_regressions.rs',
  'crates/petunia_design_document/tests/color_input_regressions.rs',
  'crates/petunia_design_document/tests/path_frame_regressions.rs',
  'crates/petunia_design_document/tests/formatter_persistence.rs',
  'crates/petunia_design_document/tests/appearance_integrity.rs',
  'crates/petunia_design_document/tests/modifier_frames.rs',
  'crates/petunia_design_geometry/tests/correctness_regressions.rs',
  'crates/petunia_design_io/tests/package_integrity.rs',
  'crates/petunia_design_io/tests/pdf_pages.rs',
  'crates/petunia_design_jobs/tests/lifecycle.rs',
  'crates/petunia_design_raster/tests/correctness_regressions.rs',
  'crates/petunia_design_render/tests/correctness_regressions.rs',
  'crates/petunia_design_testkit/src/bin/canvas_benchmark.rs',
];
const reference = {
  contract: 'ADR-003', related_contracts: ['ADR-002'], scope: 'Milestone Required',
  schema_version: Number(read(files[0]).match(/NATIVE_SCHEMA_VERSION: u32 = (\d+)/)[1]),
  rust_toolchain: read('rust-toolchain.toml').match(/channel = "([^"]+)"/)[1],
  gui: { runtime: 'Freya/Skia', version: read('apps/petunia-design/Cargo.toml').match(/freya\s*=\s*"([^"]+)"/)?.[1] ?? 'see Cargo.lock' },
  sources: files.map((file) => ({ path: file, sha256: crypto.createHash('sha256').update(read(file)).digest('hex') })),
  tests: tests.map((file) => ({ path: file, count: [...read(file).matchAll(/#\[test\]/g)].length })),
  validation_results: 'checks.json',
  integration_regressions: [{ path: 'crates/petunia_design_shell/tests/tools_test.rs', cases: ['perspective_drag_on_rotated_object_preserves_untouched_world_corners'] }],
  limitations: ['Only immutable path buffers are shared; document containers and image resources still copy.', 'Full spatial opacity consumption by preview/export is pending; the sampling API preserves local frames.', 'World-axis-aligned crop on a rotated object requires a polygon mask, which is not implemented.', 'The benchmark measures snapshot queries, not painted frames.', 'MVP and V1 release gates remain open.'],
};
const output = path.join(root, 'docs/public/implementation/contracts.json');
const content = JSON.stringify(reference, null, 2) + '\n';
if (process.argv.includes('--check')) {
  if (!fs.existsSync(output) || fs.readFileSync(output, 'utf8') !== content) {
    console.error('implementation contract reference is stale; run node scripts/generate-implementation-reference.js');
    process.exit(1);
  }
} else {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, content);
}
