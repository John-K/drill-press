Drill-Press
==========

![Crates.io](https://img.shields.io/crates/v/drill-press?style=flat-square&logo=rust) ![License](https://img.shields.io/crates/l/drill-press?style=flat-square) ![Unsafe](https://img.shields.io/badge/unsafe-very%20yes-important?style=flat-square) ![Maintenance](https://img.shields.io/maintenance/yes/2025?style=flat-square)

A simple, cross platform crate for finding the locations of holes in sparse files.

Forked from Nathan McCarty's [hole_punch](https://docs.rs/hole-punch) ([git](https://gitlab.com/asuran-rs/hole-punch))

Currently supports Unix-like platforms that support the `SEEK_HOLE` and `SEEK_DATA` commands on `lseek`, as well as windows.

The operating systems that currently support filesystem-level sparsity information are:

1.	Linux
2.	Android
3.	FreeBSD
4.	Windows
5.	MacOS

These are currently implemented with a compile time switch, and `SparseFile::scan_chunks` will always immediately return with a `ScanError::UnsupportedPlatform` error on platforms not on this list.

Usage
-----

```rust
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use drill_press::*;

if let Ok(mut file) = File::open("README.md") {
    let segments = file.scan_chunks().expect("Unable to scan chunks");
    for segment in segments.data() {
        let start = segment.start;
        let length = segment.end - segment.start;
        file.seek(SeekFrom::Start(start));
        let chunk = (&mut file).take(length);
    }
}
```

License
-------

Drill-Press is distributed under your choice of the MIT license, or Apache 2.0.

