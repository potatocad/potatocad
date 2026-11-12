//! STL output with no allocator: binary STL into a caller's byte buffer, ASCII STL into any `core::fmt::Write`.

use core::fmt::{self, Write};

/// One facet: an outward unit normal and three vertices in counter-clockwise order seen from outside.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
    /// Outward unit normal.
    pub normal: [f32; 3],
    /// The three corners.
    pub vertices: [[f32; 3]; 3],
}

/// Why a binary STL could not be written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StlError {
    /// The buffer holds fewer bytes than [`binary_len`] says the triangles need.
    BufferTooSmall {
        /// Bytes the STL needs.
        needed: usize,
        /// Bytes the buffer has.
        available: usize,
    },
    /// More triangles than a binary STL's 32-bit count can hold.
    TooManyTriangles,
}

/// The exact size in bytes of a binary STL with `triangles` facets: an 80-byte header, a 4-byte count and 50 bytes
/// a facet. `None` if it would overflow `usize`.
pub const fn binary_len_checked(triangles: usize) -> Option<usize> {
    match triangles.checked_mul(50) {
        Some(body) => body.checked_add(84),
        None => None,
    }
}

/// The exact size in bytes of a binary STL with `triangles` facets (saturating on overflow).
pub const fn binary_len(triangles: usize) -> usize {
    match binary_len_checked(triangles) {
        Some(n) => n,
        None => usize::MAX,
    }
}

/// Write a binary STL into `out`, returning the bytes written. `header` fills the 80-byte header (truncated, then
/// padded with spaces). Nothing is written when the buffer is too small.
pub fn write_binary(
    triangles: &[Triangle],
    header: &[u8],
    out: &mut [u8],
) -> Result<usize, StlError> {
    let count = u32::try_from(triangles.len()).map_err(|_| StlError::TooManyTriangles)?;
    let needed = binary_len_checked(triangles.len()).ok_or(StlError::TooManyTriangles)?;
    if out.len() < needed {
        return Err(StlError::BufferTooSmall {
            needed,
            available: out.len(),
        });
    }
    let (head, rest) = out.split_at_mut(80);
    head.fill(b' ');
    let h = header.len().min(80);
    head[..h].copy_from_slice(&header[..h]);
    let (count_bytes, mut body) = rest.split_at_mut(4);
    count_bytes.copy_from_slice(&count.to_le_bytes());
    for t in triangles {
        let (facet, next) = body.split_at_mut(50);
        let mut at = 0;
        for v in core::iter::once(&t.normal).chain(t.vertices.iter()) {
            for c in v {
                facet[at..at + 4].copy_from_slice(&c.to_le_bytes());
                at += 4;
            }
        }
        facet[48..50].copy_from_slice(&0u16.to_le_bytes());
        body = next;
    }
    Ok(needed)
}

/// Write an ASCII STL named `name` to `out`.
pub fn write_ascii<W: Write>(triangles: &[Triangle], name: &str, out: &mut W) -> fmt::Result {
    writeln!(out, "solid {name}")?;
    for t in triangles {
        let [nx, ny, nz] = t.normal;
        writeln!(out, "  facet normal {nx:e} {ny:e} {nz:e}")?;
        writeln!(out, "    outer loop")?;
        for [x, y, z] in t.vertices {
            writeln!(out, "      vertex {x:e} {y:e} {z:e}")?;
        }
        writeln!(out, "    endloop")?;
        writeln!(out, "  endfacet")?;
    }
    writeln!(out, "endsolid {name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: Triangle = Triangle {
        normal: [0.0, 0.0, 1.0],
        vertices: [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]],
    };

    #[test]
    fn binary_layout_is_exact() {
        let mut buf = [0u8; 184];
        let n = write_binary(&[T, T], b"potato", &mut buf).unwrap();
        assert_eq!(n, 184);
        assert_eq!(&buf[..6], b"potato");
        assert!(buf[6..80].iter().all(|&b| b == b' '));
        assert_eq!(u32::from_le_bytes([buf[80], buf[81], buf[82], buf[83]]), 2);
        assert_eq!(
            f32::from_le_bytes([buf[92], buf[93], buf[94], buf[95]]),
            1.0
        ); // normal z
        assert_eq!(
            f32::from_le_bytes([buf[108], buf[109], buf[110], buf[111]]),
            10.0
        ); // second vertex x
        assert_eq!(&buf[132..134], &[0, 0]);
    }

    #[test]
    fn a_small_buffer_is_refused_untouched() {
        let mut buf = [7u8; 100];
        assert_eq!(
            write_binary(&[T], b"", &mut buf),
            Err(StlError::BufferTooSmall {
                needed: 134,
                available: 100
            })
        );
        assert!(buf.iter().all(|&b| b == 7));
    }

    #[test]
    fn sizes() {
        assert_eq!(binary_len(0), 84);
        assert_eq!(binary_len(3), 234);
        assert_eq!(binary_len_checked(usize::MAX), None);
    }

    struct S {
        s: [u8; 512],
        n: usize,
    }
    impl Write for S {
        fn write_str(&mut self, t: &str) -> fmt::Result {
            let b = t.as_bytes();
            let end = self
                .n
                .checked_add(b.len())
                .filter(|&e| e <= self.s.len())
                .ok_or(fmt::Error)?;
            self.s[self.n..end].copy_from_slice(b);
            self.n = end;
            Ok(())
        }
    }

    #[test]
    fn ascii_round_shape() {
        let mut s = S { s: [0; 512], n: 0 };
        write_ascii(&[T], "potato", &mut s).unwrap();
        let text = core::str::from_utf8(&s.s[..s.n]).unwrap();
        assert!(text.starts_with("solid potato\n  facet normal 0e0 0e0 1e0\n"));
        assert!(text.ends_with("endsolid potato\n"));
        assert_eq!(text.matches("vertex").count(), 3);
    }
}
