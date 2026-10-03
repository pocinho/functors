# Phase 1 Performance Baseline

The initial baseline uses the ignored test
`tests::reports_document_edit_and_layout_baseline`:

```text
cargo test reports_document_edit_and_layout_baseline -- --ignored --nocapture
```

The test measures 100 representative insertions into a 10,000-line Rope-backed
document, 100 deterministic frame builds, and 100 Vello scene builds for the
same document. It reports elapsed wall-clock time without asserting a
threshold, because the result is machine and workload dependent. These
measurements are for comparison before introducing further caching, background
work, or storage changes.

Baseline captured on 2026-10-02:

```text
100 edits=4.0842ms, 100 layouts=50.0854ms
```

The scene-build measurement is reported in subsequent baseline runs after the
Vello scene adapter is active.

Run captured on 2026-10-03 after adding the Vello scene measurement:

```text
100 edits=4.0814ms, 100 layouts=8.0622761s, 100 vello scenes=760us
```