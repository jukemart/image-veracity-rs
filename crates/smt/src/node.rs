use std::fmt::{Debug, Display, Formatter};

type Byte = u8;

/// ID identifies a node of a Merkle tree. It is a bit string that counts the
/// node down from the tree root, i.e. 0 and 1 bits represent going to the left
/// or right child correspondingly.
///
/// ID is immutable, comparable, and can be used as a Golang map key. It also
/// incurs zero memory allocations in transforming methods like Prefix and
/// Sibling.
///
/// The internal structure of ID is driven by its use-cases:
///   - To make ID objects immutable and comparable, the Golang string type is
///     used for storing the bit string bytes.
///   - To make Sibling and Prefix operations fast, the last byte is stored
///     separately from the rest of the bytes, so that it can be "amended".
///   - To make ID objects comparable, there is only one (canonical) way to encode
///     an ID. For example, if the last byte is used partially, its unused bits
///     are always unset. See invariants next to field definitions below.
///
/// Constructors and methods of ID make sure its invariants are always met.
///
/// For example, an 11-bit node ID [1010,1111,001] is structured as follows:
/// - path string contains 1 byte, which is [1010,1111].
/// - last byte is [0010,0000]. Note the unset lower 5 bits.
/// - bits is 3, so effectively only the upper 3 bits [001] of last are used.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ID {
    path: Box<[u8]>,
    last: Byte,
    // Invariant: Lowest (8-bits) bits of the last byte are unset.
    bits: u8, // Invariant: 1 <= bits <= 8, or bits == 0 for the empty ID.
}

impl ID {
    /// NewID creates a node ID from the given path bytes truncated to the specified
    /// number of bits if necessary. Panics if the number of bits is more than the
    /// byte string contains.
    pub fn new_id(path: &[u8], bits: usize) -> ID {
        if bits == 0 {
            return ID::default();
        } else if bits > path.len() * 8 {
            panic!("NewID: bits {} > {}", bits, path.len() * 8)
        }

        let (bytes, tail_bits) = split(bits);
        ID::new_masked_id(&path[..bytes], &path[bytes], tail_bits)
    }

    /// newMaskedID constructs a node ID ensuring its invariants are met. The last
    /// byte is masked so that the given number of upper bits are in use, and the
    /// others are unset.
    fn new_masked_id(path: &[u8], last: &Byte, bits: u8) -> ID {
        let mut last = *last;
        // last &= ^byte(1<<(8-bits) - 1) // Unset the unused bits.
        // let shift = 1 << (8 - bits); // can overflow and panic
        let shift = safe_shift_left(1, 8 - bits);
        let new_byte = if shift == 0 { 255 } else { shift - 1 };
        last &= !new_byte;

        ID {
            path: Box::from(path),
            last,
            bits,
        }
    }

    /// NewIDWithLast creates a node ID from the given path bytes and the additional
    /// last byte, of which only the specified number of most significant bits is
    /// used. The number of bits must be between 1 and 8, and can be 0 only if the
    /// path bytes string is empty; otherwise the function panics.
    pub fn new_id_with_last(path: &[u8], last: Byte, bits: u8) -> ID {
        if bits > 8 {
            panic!("NewIDWithLast: bits {bits} > 8")
        } else if bits == 0 && !path.is_empty() {
            panic!("NewIdWithLast: bits=0, but path is not empty")
        }
        ID::new_masked_id(path, &last, bits)
    }

    /// bit_length returns the length of the ID in bits
    pub fn bit_length(&self) -> usize {
        self.path.len() * 8 + self.bits as usize
    }

    /// Prefix reutrns the prefix of the node ID with the given number of bits.
    pub fn prefix(&self, bits: usize) -> ID {
        // Note: This code is very similar to NewID, and it's tempting to return
        // NewID(n.path, bits). But there is a difference: NewID expects all the
        // bytes to be in the path string, while here the last byte is not.
        if bits == 0 {
            ID::default()
        } else if bits > self.bit_length() {
            panic!("Prefix: bits {} > {}", bits, self.bit_length())
        } else {
            let (bytes, tail_bits) = split(bits);
            let last = if bytes != self.path.len() {
                self.path[bytes]
            } else {
                self.last
            };
            ID::new_masked_id(&self.path[..bytes], &last, tail_bits)
        }
    }
}

impl Display for ID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.bit_length() == 0 {
            return write!(f, "[]");
        }
        write!(f, "{:b}", self)
    }
}

impl std::fmt::Binary for ID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        // Add the paths
        for i in 0..self.path.len() {
            output.push_str(&format!("{:08b}", self.path[i]));
            output.push(' ');
        }

        // Pad the last bits
        let pad_zero: usize = self.bits as usize;
        output.push_str(&format!("{:>0pad_zero$b}", self.last >> (8 - self.bits)));
        write!(f, "[{}]", output)
    }
}

/// split returns the decomposition of an ID with the given number of bits. The
/// first int returned is the number of full bytes stored in the dynamically
/// allocated part. The second one is the number of bits in the tail byte.
fn split(bits: usize) -> (usize, u8) {
    ((bits - 1) / 8, (1 + (bits - 1) % 8) as u8)
}

/// safe_shift-left avoids overflow while left-shifting, which results in UB and is sometimes abused
/// in other non-Rust languages
/// N.B. may need performance adjustments since we know the sizes before hand
fn safe_shift_left(mut input: i32, mut shift_by: u8) -> u8 {
    while shift_by != 0 {
        let local_shift = shift_by % 32;
        input <<= local_shift;
        shift_by -= local_shift;
    }

    input as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! new_id_with_last_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                const TEST_BYTES: &[u8; 4] = b"\x0A\x0B\x0C\xFA";
                let (path, length, last, bits) = $value;
                let id = ID::new_id(TEST_BYTES, length);
                let got = ID::new_id_with_last(path, last, bits);
                let want = id;

                assert_eq!(want, got, "NewIDWithLast: got {:?}, want {:?}", got, want);
            }
        )*
        }
    }
    new_id_with_last_tests! {
        id_empty_0_0_0: (&[], 0, 0, 0),
        id_empty_0_123_0: (&[], 0, 123, 0),
        id_empty_1_0_1: (&[], 1, 0, 1),
        id_empty_1_123_1: (&[], 1, 123, 1),
        id_empty_4_0_4: (&[], 4, 0, 4),
        id_empty_5_a_5: (&[], 5, 0xA, 5),
        id_empty_5_f_5: (&[], 5, 0xF, 5),
        id_empty_7_b_7: (&[], 7, 0xB, 7),
        id_first_byte_9_0_1: (&TEST_BYTES[..1], 9, 0, 1),
        id_first_byte_13_a_5: (&TEST_BYTES[..1], 13, 0xA, 5),
        id_two_bytes_24_c_8: (&TEST_BYTES[..2], 24, 0xC, 8),
        id_three_bytes_31_fb_7: (&TEST_BYTES[..3], 31, 0xFB, 7),
        id_three_bytes_31_fa_7: (&TEST_BYTES[..3], 31, 0xFA, 7),
        id_three_bytes_32_fa_8: (&TEST_BYTES[..3], 32, 0xFA, 8),
    }

    #[test]
    fn id_comparison() {
        const TEST_BYTES: &[u8; 7] = b"\x0A\x0B\x0C\x0A\x0B\x0C\x01";
        let test_cases = vec![
            (
                "all-same",
                ID::new_id(TEST_BYTES, 56),
                ID::new_id(TEST_BYTES, 56),
                true,
            ),
            (
                "same-bytes",
                ID::new_id(&TEST_BYTES[..3], 24),
                ID::new_id(&TEST_BYTES[3..6], 24),
                true,
            ),
            (
                "same-bits1",
                ID::new_id(&TEST_BYTES[..4], 25),
                ID::new_id(&TEST_BYTES[3..], 25),
                true,
            ),
            (
                "same-bits2",
                ID::new_id(&TEST_BYTES[..4], 28),
                ID::new_id(&TEST_BYTES[3..], 28),
                true,
            ),
            (
                "diff-bits",
                ID::new_id(&TEST_BYTES[..4], 29),
                ID::new_id(&TEST_BYTES[3..], 29),
                false,
            ),
            (
                "diff-len",
                ID::new_id(TEST_BYTES, 56),
                ID::new_id(TEST_BYTES, 55),
                false,
            ),
            (
                "diff-bytes",
                ID::new_id(TEST_BYTES, 56),
                ID::new_id(TEST_BYTES, 48),
                false,
            ),
        ];

        for (desc, id1, id2, want) in test_cases {
            let equality_check = id1 == id2;
            assert_eq!(
                equality_check, want,
                "{}: (id1==id2) is {}, want {}. Values:\n(id1: {:#?},\nid2: {:#?})",
                desc, equality_check, want, id1, id2
            );
        }
    }

    #[test]
    fn id_to_string() {
        const TEST_BYTES: &[u8; 3] = &[5_u8, 1_u8, 127_u8];

        let test_cases = vec![
            (0, "[]"),
            (1, "[0]"),
            (4, "[0000]"),
            (6, "[000001]"),
            (8, "[00000101]"),
            (16, "[00000101 00000001]"),
            (21, "[00000101 00000001 01111]"),
            (24, "[00000101 00000001 01111111]"),
        ];

        for (bits, want) in test_cases {
            let id = ID::new_id(TEST_BYTES, bits);
            let got = id.to_string();
            assert_eq!(got, want, "String: got {}, want {}", got, want);
        }
    }

    #[test]
    fn id_prefix() {
        const TEST_BYTES: &[u8; 3] = b"\x0A\x0B\x0C";

        let test_cases = vec![
            (ID::new_id(TEST_BYTES, 24), 0, ID::default()),
            (ID::new_id(TEST_BYTES, 24), 1, ID::new_id(TEST_BYTES, 1)),
            (ID::new_id(TEST_BYTES, 24), 2, ID::new_id(TEST_BYTES, 2)),
            (ID::new_id(TEST_BYTES, 24), 5, ID::new_id(TEST_BYTES, 5)),
            (ID::new_id(TEST_BYTES, 24), 8, ID::new_id(TEST_BYTES, 8)),
            (ID::new_id(TEST_BYTES, 24), 15, ID::new_id(TEST_BYTES, 15)),
            (ID::new_id(TEST_BYTES, 24), 24, ID::new_id(TEST_BYTES, 24)),
            (ID::new_id(TEST_BYTES, 21), 15, ID::new_id(TEST_BYTES, 15)),
        ];

        for (id, bits, want) in test_cases {
            let got = id.prefix(bits);
            assert_eq!(
                got, want,
                "Prefix bits={}: got {}, want {}",
                bits, got, want
            );
        }
    }
}
