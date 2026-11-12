# PotatoCAD

CAD that runs on a potato. PotatoCAD is a small `no_std` core for parametric CAD on microcontrollers (an ESP32 is the
first target), with the same code running on a desktop too.

This first release is the piece every part ends in: an **STL writer** that needs no allocator and no standard library.
Give it triangles and a byte buffer (binary STL) or any `core::fmt::Write` (ASCII STL).

```rust
use potatocad::stl::{Triangle, write_binary, binary_len};

let tris = [Triangle { normal: [0.0, 0.0, 1.0], vertices: [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]] }];
let mut buf = [0u8; 134];
assert_eq!(binary_len(tris.len()), 134);
let n = write_binary(&tris, b"potato", &mut buf).unwrap();
assert_eq!(n, 134);
```

The model, sketches, extrude, revolve and the software renderer follow.

MIT licensed.
