# Observatory runtime asset measurement

Measured from fresh production `web/dist/` after `bun run build`.

| Payload | Current build |
| --- | ---: |
| Runtime raster images | 249,596 bytes (4 files) |
| Complete `web/dist/` | 786,349 bytes |
| Runtime raster ceiling | 6,291,456 bytes |

Purpose-named runtime derivatives:

- `assets/images/index-observation-field.webp` — 95,710 bytes
- `assets/images/index-plate-sun.webp` — 65,168 bytes
- `assets/images/index-plate-face.webp` — 73,302 bytes
- `assets/images/index-plate-star-system.webp` — 15,416 bytes

All four files are local, referenced by `IndexView.vue`, and requested through embedded
same-origin dashboard lane. Browser-health checks report no failed or external requests.

Verification:

```text
runtime images allowed: 4 file, 249596 / 6291456 bytes
```

`index-orbit.webp` and source PNGs `main-panel-background.png`, `abstract-sun.png`,
`abstract-face.png`, and `abstract-star-system.png` do not exist under `web/dist/`.
