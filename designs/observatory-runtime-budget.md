# Observatory runtime asset measurement

Measured from fresh production `web/dist/` after `bun run build`.

| Payload | Current build |
| --- | ---: |
| Runtime raster images | 249,596 bytes (4 files) |
| Complete `web/dist/` | 800,595 bytes |
| Runtime raster ceiling | 6,291,456 bytes |

Shared observatory runtime derivatives:

- `assets/images/observatory-field.webp` — 95,710 bytes
- `assets/images/observatory-plate-sun.webp` — 65,168 bytes
- `assets/images/observatory-plate-face.webp` — 73,302 bytes
- `assets/images/observatory-plate-star-system.webp` — 15,416 bytes

Aggregate remains 249,596 bytes. All four files are local and requested through embedded
same-origin dashboard lane. Field derivative serves index masthead plus selected-document
arrival. Three plate derivatives remain index lower-strip assets; scope pre-document gallery
is removed. Browser-health checks report no failed or external requests.

Verification:

```text
runtime images allowed: 4 file, 249596 / 6291456 bytes
```

`index-orbit.webp`, old `index-*` aliases, duplicate `scope-*` aliases, and source PNGs
`main-panel-background.png`, `abstract-sun.png`, `abstract-face.png`, and
`abstract-star-system.png` do not exist under `web/dist/`.
