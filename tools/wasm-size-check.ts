#!/usr/bin/env ts-node
/**
 * SC-W5-040 – WASM size budget enforcement for release builds.
 *
 * Checks that the compiled WASM artifact stays within the configured size
 * budget. Run as part of CI (or locally before release) to prevent
 * accidental bloat.
 *
 * Usage:
 *   npx ts-node tools/wasm-size-check.ts <path-to-wasm> [max-size-bytes]
 *
 * Default max size: 64 KB (65536 bytes)
 */

const fs = require('fs');
const path = require('path');

const DEFAULT_MAX_SIZE = 65_536; // 64 KB
const WARNING_THRESHOLD = 0.90;  // warn at 90% of budget

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1048576).toFixed(2)} MB`;
}

function main(): void {
  const wasmPath = process.argv[2];
  const maxSizeBytes = parseInt(process.argv[3], 10) || DEFAULT_MAX_SIZE;

  if (!wasmPath) {
    console.error('Usage: ts-node tools/wasm-size-check.ts <path-to-wasm> [max-size-bytes]');
    process.exit(1);
  }

  if (!fs.existsSync(wasmPath)) {
    console.error(`❌ WASM file not found: ${wasmPath}`);
    process.exit(1);
  }

  const stats = fs.statSync(wasmPath);
  const size = stats.size;

  console.log(`WASM artifact: ${path.basename(wasmPath)}`);
  console.log(`Size:         ${formatBytes(size)}`);
  console.log(`Budget:       ${formatBytes(maxSizeBytes)}`);
  console.log(`Utilisation:  ${((size / maxSizeBytes) * 100).toFixed(1)}%`);

  if (size > maxSizeBytes) {
    console.error(`❌ FAIL: WASM size ${formatBytes(size)} exceeds budget of ${formatBytes(maxSizeBytes)}`);
    process.exit(1);
  }

  if (size > maxSizeBytes * WARNING_THRESHOLD) {
    console.warn(`⚠️  WARN: WASM size is within ${((1 - WARNING_THRESHOLD) * 100).toFixed(0)}% of the budget`);
  }

  console.log('✅ PASS: WASM is within size budget');
  process.exit(0);
}

main();
