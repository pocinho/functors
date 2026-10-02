# Phase 1 Performance Baseline

The initial baseline uses the ignored test
`tests::reports_document_edit_and_layout_baseline`:

```text
cargo test reports_document_edit_and_layout_baseline -- --ignored --nocapture
```

The test measures 100 representative insertions into a 10,000-line Rope-backed
document and 100 deterministic frame builds for the same document. It reports
elapsed wall-clock time without asserting a threshold, because the result is
machine and workload dependent. These measurements are for comparison before
introducing caching, background work, or storage changes.

Baseline captured on 2026-10-02:

```text
100 edits=4.0842ms, 100 layouts=50.0854ms
```