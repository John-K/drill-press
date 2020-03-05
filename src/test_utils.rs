use super::*;

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use tempfile::tempfile;

use quickcheck::{Arbitrary, Gen};
use rand::Rng;
use std::collections::HashSet;

const MAX_LENGTH: u64 = 10_000_000;
const MAX_SPLITS: u64 = 10;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Tag {
    Data(u64),
    Hole(u64),
    End(u64),
}
impl Tag {
    fn offset(&self) -> u64 {
        match self {
            Tag::Data(x) | Tag::Hole(x) | Tag::End(x) => *x,
        }
    }
}
#[derive(Clone)]
pub struct SparseDescription(Vec<Segment>);

impl SparseDescription {
    pub fn to_file(&self) -> File {
        // First, make our file
        let mut file = tempfile().expect("Unable to create temp file");
        // Iterate through the SparseDescription
        for segment in &self.0 {
            // Only proceed if this is a data segment
            if let SegmentType::Data = segment.segment_type {
                // Seek to the start of the segment
                file.seek(SeekFrom::Start(segment.start))
                    .expect("Unable to seek in file");
                let len = segment.end - segment.start;
                // Create a buffer of 1s to read from
                let buffer = vec![1_u8; len as usize];
                // write those ones to the file
                file.write_all(&buffer[..])
                    .expect("Unable to write bytes to file");
            }
        }
        file
    }
}

impl Arbitrary for SparseDescription {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        // Select a random length between 0 and MAX_LENGTH
        let length = g.gen_range(0, MAX_LENGTH);
        // Create a vector of tags
        let mut tags: Vec<Tag> = Vec::new();
        // Select a random number of splits
        let splits = g.gen_range(0, MAX_SPLITS + 1);
        let starts_with_hole: bool = g.gen();
        // Generate splits number of splits
        // not yet sorted
        // Does not yet have first or last tag
        for _ in 0..splits {
            let split_point = g.gen_range(0, length);
            tags.push(Tag::Data(split_point));
        }
        // Add start and end tags
        if starts_with_hole {
            tags.push(Tag::Hole(0));
        } else {
            tags.push(Tag::Data(0));
        }
        tags.push(Tag::End(length));
        // elminate duplicates.
        let set = tags.drain(..).collect::<HashSet<_>>();
        tags.extend(set.into_iter());
        // Sort the tags
        tags.sort_by_key(Tag::offset);
        // Modify each tag so it alternates between hole and data
        // Don't touch the start or end tags
        for i in 1..tags.len() - 1 {
            let current = tags[i];
            let previous = tags[i - 1];
            if let Tag::Data(_) = previous {
                tags[i] = Tag::Hole(current.offset());
            }
        }

        // Process our list of start point tags into a list of segments.
        let tag_pairs = tags
            .iter()
            .copied()
            .zip(tags.iter().skip(1).copied())
            .map(|(x, y)| {
                // All these casts are valid, as the wrapper methods we use
                // around lseek will return Err rather than returning a value
                // less than 0
                match x {
                    Tag::Data(start) => Segment {
                        segment_type: SegmentType::Data,
                        start: start as u64,
                        end: (y.offset() - 1) as u64,
                    },
                    Tag::Hole(start) => Segment {
                        segment_type: SegmentType::Hole,
                        start: start as u64,
                        end: (y.offset() - 1) as u64,
                    },
                    // End should only ever be the last element the tag vector,
                    // so it can never be the first element of a pair
                    Tag::End(_) => unreachable!(),
                }
            })
            .collect();
        SparseDescription(tag_pairs)
    }
}
