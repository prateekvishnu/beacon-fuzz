use ssz::Encode; //Decode
use ssz_derive::{Decode, Encode};

use types::{BeaconState, MainnetEthSpec, SignedBeaconBlock};

#[link(name = "nfuzz", kind = "static")]
extern "C" {
    fn nfuzz_block(
        input_ptr: *mut u8,
        input_size: usize,
        output_ptr: *mut u8,
        output_size: *mut usize,
        disable_bls: bool,
    ) -> bool;
}

#[derive(Decode, Encode)]
struct BlockTestCase {
    pub pre: BeaconState<MainnetEthSpec>,
    pub beacon_block: SignedBeaconBlock<MainnetEthSpec>,
}

use crate::debug::dump_post_state;

pub fn process_block(
    beacon: &BeaconState<MainnetEthSpec>,
    beacon_block: &SignedBeaconBlock<MainnetEthSpec>,
    post: &[u8],
    disable_bls: bool,
    debug: bool,
) -> bool {
    let mut out: Vec<u8> = vec![0 as u8; post.len()];

    // create testcase ssz struct
    let target: BlockTestCase = BlockTestCase {
        pre: beacon.clone(),
        beacon_block: beacon_block.clone(),
    };

    let ssz_bytes = target.as_ssz_bytes();
    let ssz_bytes_len = ssz_bytes.len();
    let mut inn: Vec<u8> = ssz_bytes.into();
    let input_ptr: *mut u8 = inn.as_mut_ptr();
    let input_size: usize = ssz_bytes_len as usize;
    let output_ptr: *mut u8 = out.as_mut_ptr();
    let output_size: *mut usize = &mut (post.len() as usize);

    let res = unsafe { nfuzz_block(input_ptr, input_size, output_ptr, output_size, disable_bls) };

    // If error triggered during processing, we return immediately
    if !res {
        return res;
    }

    if out != post {
        if debug {
            println!("[NIMBUS] Mismatch post");
        } else {
            // make fuzzer to crash
            panic!("[NIMBUS] Mismatch post");
        }
    }

    // dump post files for debugging
    if debug {
        dump_post_state(&post, &out);
    }

    res
}
