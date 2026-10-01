#!/usr/bin/env node
// Generates the source reference used by ADR-002. No test result is inferred.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const root = path.resolve(__dirname, '..');
const read = (name) => fs.readFileSync(path.join(root, name), 'utf8');
const files = [
  'crates/petunia_design_foundation/src/lib.rs',
  'crates/petunia_design_document/Cargo.toml',
  'crates/petunia_design_document/src/document_object.rs',
  'crates/petunia_design_document/src/document.rs',
  'crates/petunia_design_document/src/changeset.rs',
  'crates/petunia_design_document/src/validation.rs',
  'crates/petunia_design_document/src/modifiers.rs',
  'crates/petunia_design_document/src/adjustments.rs',
  'crates/petunia_design_document/src/mutator.rs',
  'crates/petunia_design_application/Cargo.toml',
  'crates/petunia_design_application/src/transaction.rs',
  'crates/petunia_design_application/src/history.rs',
  'crates/petunia_design_application/src/commands.rs',
  'crates/petunia_design_application/src/background_render.rs',
  'crates/petunia_design_application/src/session.rs',
  'apps/petunia-design/src/canvas_paint.rs',
  'apps/petunia-design/src/dock.rs',
  'crates/petunia_design_application/src/export_service.rs',
  'crates/petunia_design_render/Cargo.toml',
  'crates/petunia_design_render/src/render_scene.rs',
  'crates/petunia_design_render/src/cpu_renderer.rs',
  'crates/petunia_design_render/src/pixel_compositor.rs',
  'crates/petunia_design_jobs/src/executor.rs',
  'crates/petunia_design_jobs/src/manager.rs',
  'crates/petunia_design_io/src/package.rs',
  'crates/petunia_design_io/src/image_io.rs',
  'crates/petunia_design_io/Cargo.toml',
  'crates/petunia_design_raster/src/image_assets.rs',
  'crates/petunia_design_raster/src/image_cache.rs',
  'crates/petunia_design_raster/Cargo.toml',
  'crates/petunia_design_raster/src/tile.rs',
  'crates/petunia_design_raster/src/brush.rs',
  'crates/petunia_design_testkit/src/bin/canvas_benchmark.rs',
  'crates/petunia_design_shell/src/tools/perspective.rs',
  'crates/petunia_design_shell/src/tools/gradient.rs',
  'crates/petunia_design_shell/src/tools/photo.rs',
  'apps/petunia-design/src/actions.rs',
  'apps/petunia-design/src/dialogs.rs',
  'apps/petunia-design/src/ui_state.rs',
  'crates/petunia_design_resources/src/shell_strings.rs',
  'apps/petunia-design/src/main.rs',
];
const tests = [
  'crates/petunia_design_application/tests/correctness_regressions.rs',
  'crates/petunia_design_application/tests/transaction_regressions.rs',
  'crates/petunia_design_application/tests/background_render.rs',
  'crates/petunia_design_application/tests/image_placement.rs',
  'crates/petunia_design_document/tests/color_input_regressions.rs',
  'crates/petunia_design_document/tests/path_frame_regressions.rs',
  'crates/petunia_design_document/tests/formatter_persistence.rs',
  'crates/petunia_design_document/tests/appearance_integrity.rs',
  'crates/petunia_design_document/tests/modifier_frames.rs',
  'crates/petunia_design_document/tests/prepared_curves.rs',
  'crates/petunia_design_document/tests/shared_image_sources.rs',
  'crates/petunia_design_geometry/tests/correctness_regressions.rs',
  'crates/petunia_design_io/tests/package_integrity.rs',
  'crates/petunia_design_io/tests/pdf_pages.rs',
  'crates/petunia_design_io/tests/png_density.rs',
  'crates/petunia_design_io/tests/image_admission.rs',
  'crates/petunia_design_jobs/tests/lifecycle.rs',
  'crates/petunia_design_jobs/tests/executor.rs',
  'crates/petunia_design_raster/tests/correctness_regressions.rs',
  'crates/petunia_design_raster/tests/image_assets.rs',
  'crates/petunia_design_render/tests/correctness_regressions.rs',
  'crates/petunia_design_render/tests/vector_rendering.rs',
  'crates/petunia_design_render/tests/image_rendering.rs',
  'crates/petunia_design_testkit/src/bin/canvas_benchmark.rs',
];
const reference = {
  contract: 'ADR-005', related_contracts: ['ADR-004', 'ADR-003', 'ADR-002'], scope: 'Milestone Required',
  schema_version: Number(read(files[0]).match(/NATIVE_SCHEMA_VERSION: u32 = (\d+)/)[1]),
  rust_toolchain: read('rust-toolchain.toml').match(/channel = "([^"]+)"/)[1],
  gui: { runtime: 'Freya/Skia', version: read('apps/petunia-design/Cargo.toml').match(/freya\s*=\s*"([^"]+)"/)?.[1] ?? 'see Cargo.lock' },
  sources: files.map((file) => ({ path: file, sha256: crypto.createHash('sha256').update(read(file)).digest('hex') })),
  tests: tests.map((file) => ({ path: file, count: [...read(file).matchAll(/#\[test\]/g)].length })),
  validation_results: 'checks.json',
  current_validation_status: 'not_run; user deferred validation until all MVP features are implemented',
  integration_regressions: [{ path: 'crates/petunia_design_shell/tests/tools_test.rs', cases: ['perspective_drag_on_rotated_object_preserves_untouched_world_corners'] }],
  limitations: ['Immutable paths/encoded sources are shared and decoded display pyramids are cached; document containers/binary resources still need COW/packaging.', 'Images and spatial opacity are implemented in CPU/PNG; GUI uses the image cache/transform/opacity adapter but full scene parity and async cold preparation remain pending. Current implementation is unvalidated.', 'World-axis-aligned crop on a rotated object requires a polygon mask, which is not implemented.', 'The benchmark measures snapshot queries, not painted frames.', 'MVP and V1 release gates remain open.'],
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
