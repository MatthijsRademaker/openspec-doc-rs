# Observatory runtime asset measurement

Measured from same checkout and release profile. Baseline used original production `web/dist/`
with all eleven source PNGs, followed by forced `openspec-doc-server` recompilation so
`rust-embed` could not reuse stale output. After measurement used fresh observatory build and
the same forced release recompilation.

| Payload | Before | After | Reduction |
| --- | ---: | ---: | ---: |
| Runtime raster images | 24,820,959 bytes (11 files) | 13,378 bytes (1 file) | 24,807,581 bytes (99.95%) |
| Complete `web/dist/` | 25,208,683 bytes (23 files) | 499,417 bytes (17 files) | 24,709,266 bytes (98.02%) |
| Release embedded binary | 28,549,360 bytes | 3,649,232 bytes | 24,900,128 bytes (87.22%) |

Runtime image: `assets/images/index-orbit.webp`, 720×540, derived from
`designs/visual-language/abstract-star-system.png` for bounded index masthead slot.

Verification:

```text
runtime images allowed: 1 file, 13378 / 6291456 bytes
```

Neither `design-system.png` nor `dashboard-mockup.png` exists anywhere under `web/dist/`.
