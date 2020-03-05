//! lseek based implemenation that uses `SEEK_DATA` and `SEEK_HOLE` to
//! reconstruct which segements of the file are data or holes
use super::*;

use std::fs::File;
use std::io::Error;
use std::os::unix::io::AsRawFd;

use libc::{lseek, SEEK_DATA, SEEK_END, SEEK_HOLE, SEEK_SET};

impl SparseFile for File {
    fn scan_chunks(&mut self) -> std::result::Result<std::vec::Vec<Segment>, ScanError> {
        // Create our output vec
        let mut holes: Vec<Segment> = Vec::new();
        // Extract the raw fd from the file
        let fd = self.as_raw_fd();
        let end;
        unsafe {
            // use lseek to find the end of the file
            end = lseek(fd, 0, SEEK_END);
            if end < 0 {
                return Err(ScanError::from(Error::last_os_error()));
            }
            // use lseek to reset the cursor to the start of the file
            let offset = lseek(fd, 0, SEEK_SET);
            if offset < 0 {
                return Err(ScanError::from(Error::last_os_error()));
            }
            // Find the first hole
            let mut last_hole_start = lseek(fd, 0, SEEK_HOLE);
            if last_hole_start < 0 {
                return Err(ScanError::from(Error::last_os_error()));
            }
            // Go through the file and create the holes list
            while last_hole_start < end {
                // Find the next data segement
                let next_data_start = lseek(fd, last_hole_start + 1, SEEK_DATA);
                if next_data_start < 0 {
                    // If we are here, we can reasonably assume we have access
                    // to the file, as we have completed several writes. For
                    // now, we will just assume we have run out of data
                    // segements and return.
                    // FIXME: Stop assuming and actually check errno
                    holes.push(Segment {
                        segment_type: SegmentType::Hole,
                        start: last_hole_start as u64,
                        end: end as u64,
                    });
                    break;
                }
                // Describe the hole
                holes.push(Segment {
                    segment_type: SegmentType::Hole,
                    // We can safely do these casts since we verified the values
                    // are non-negative
                    start: last_hole_start as u64,
                    end: next_data_start as u64 - 1,
                });
                // find the next hole
                last_hole_start = lseek(fd, next_data_start + 1, SEEK_HOLE);
                if last_hole_start < 0 {
                    return Err(ScanError::from(Error::last_os_error()));
                }
            }
        }
        // If holes is empty, the file is empty, check to see if the file is empty, and if
        // it is, return a empty vector. Otherwise, return just a data chunk
        if holes.is_empty() {
            if end <= 0 {
                Ok(holes)
            } else {
                Ok(vec![Segment {
                    segment_type: SegmentType::Data,
                    start: 0,
                    // This cast is valid, as we would have thrown an Err if end was negative
                    end: end as u64,
                }])
            }
        } else {
            let mut output = Vec::new();
            // figure out if the first segement is a hole
            // Insert a data segment if it isnt
            let mut last_end = 0;
            if holes[0].start != 0 {
                output.push(Segment {
                    segment_type: SegmentType::Data,
                    start: 0,
                    end: holes[0].start - 1,
                });
                last_end = holes[0].end - 1;
            }
            for hole in holes {
                // Figure out if there is a data segement in between this hole and the last
                if last_end == 0 || hole.start > last_end + 1 {
                    output.push(Segment {
                        segment_type: SegmentType::Data,
                        start: last_end + 1,
                        end: hole.start - 1,
                    });
                }
                output.push(hole)
            }
            // Figure out if there is a data segement at the end that needs to be added
            if (output[output.len() - 1].end as i64) < end {
                output.push(Segment {
                    segment_type: SegmentType::Data,
                    start: output[output.len() - 1].end + 1,
                    end: end as u64,
                });
            }
            Ok(output)
        }
    }
}
