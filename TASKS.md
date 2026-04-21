# bytes — ETNA Tasks

Total tasks: 20

ETNA tasks are **mutation/property/witness triplets**. Each row below is one runnable task.

## Task Index

| Task | Variant | Framework | Property | Witness | Command |
|------|---------|-----------|----------|---------|---------|
| 001  | `sign_extend_zero_nbytes_acd1e0f_1` | proptest | `property_get_int_zero_nbytes` | `witness_get_int_zero_nbytes_case_eight_zero_bytes` | `cargo run --release --bin etna --features etna -- proptest GetIntZeroNbytes` |
| 002  | `sign_extend_zero_nbytes_acd1e0f_1` | quickcheck | `property_get_int_zero_nbytes` | `witness_get_int_zero_nbytes_case_eight_zero_bytes` | `cargo run --release --bin etna --features etna -- quickcheck GetIntZeroNbytes` |
| 003  | `sign_extend_zero_nbytes_acd1e0f_1` | crabcheck | `property_get_int_zero_nbytes` | `witness_get_int_zero_nbytes_case_eight_zero_bytes` | `cargo run --release --bin etna --features etna -- crabcheck GetIntZeroNbytes` |
| 004  | `sign_extend_zero_nbytes_acd1e0f_1` | hegel | `property_get_int_zero_nbytes` | `witness_get_int_zero_nbytes_case_eight_zero_bytes` | `cargo run --release --bin etna --features etna -- hegel GetIntZeroNbytes` |
| 005  | `get_int_sign_extension_79fb853_1` | proptest | `property_get_int_sign_extension` | `witness_get_int_sign_extension_case_four_ff_bytes` | `cargo run --release --bin etna --features etna -- proptest GetIntSignExtension` |
| 006  | `get_int_sign_extension_79fb853_1` | quickcheck | `property_get_int_sign_extension` | `witness_get_int_sign_extension_case_four_ff_bytes` | `cargo run --release --bin etna --features etna -- quickcheck GetIntSignExtension` |
| 007  | `get_int_sign_extension_79fb853_1` | crabcheck | `property_get_int_sign_extension` | `witness_get_int_sign_extension_case_four_ff_bytes` | `cargo run --release --bin etna --features etna -- crabcheck GetIntSignExtension` |
| 008  | `get_int_sign_extension_79fb853_1` | hegel | `property_get_int_sign_extension` | `witness_get_int_sign_extension_case_four_ff_bytes` | `cargo run --release --bin etna --features etna -- hegel GetIntSignExtension` |
| 009  | `chain_remaining_saturating_2428c15_1` | proptest | `property_chain_remaining_saturating` | `witness_chain_remaining_saturating_case_max_plus_one` | `cargo run --release --bin etna --features etna -- proptest ChainRemainingSaturating` |
| 010  | `chain_remaining_saturating_2428c15_1` | quickcheck | `property_chain_remaining_saturating` | `witness_chain_remaining_saturating_case_max_plus_one` | `cargo run --release --bin etna --features etna -- quickcheck ChainRemainingSaturating` |
| 011  | `chain_remaining_saturating_2428c15_1` | crabcheck | `property_chain_remaining_saturating` | `witness_chain_remaining_saturating_case_max_plus_one` | `cargo run --release --bin etna --features etna -- crabcheck ChainRemainingSaturating` |
| 012  | `chain_remaining_saturating_2428c15_1` | hegel | `property_chain_remaining_saturating` | `witness_chain_remaining_saturating_case_max_plus_one` | `cargo run --release --bin etna --features etna -- hegel ChainRemainingSaturating` |
| 013  | `partialord_bytes_reversed_939a5ed_1` | proptest | `property_partialord_bytes_reversed` | `witness_partialord_bytes_reversed_case_lhs_less` | `cargo run --release --bin etna --features etna -- proptest PartialOrdBytesReversed` |
| 014  | `partialord_bytes_reversed_939a5ed_1` | quickcheck | `property_partialord_bytes_reversed` | `witness_partialord_bytes_reversed_case_lhs_less` | `cargo run --release --bin etna --features etna -- quickcheck PartialOrdBytesReversed` |
| 015  | `partialord_bytes_reversed_939a5ed_1` | crabcheck | `property_partialord_bytes_reversed` | `witness_partialord_bytes_reversed_case_lhs_less` | `cargo run --release --bin etna --features etna -- crabcheck PartialOrdBytesReversed` |
| 016  | `partialord_bytes_reversed_939a5ed_1` | hegel | `property_partialord_bytes_reversed` | `witness_partialord_bytes_reversed_case_lhs_less` | `cargo run --release --bin etna --features etna -- hegel PartialOrdBytesReversed` |
| 017  | `slice_ref_empty_f330ef6_1` | proptest | `property_slice_ref_empty` | `witness_slice_ref_empty_case_nonempty_source` | `cargo run --release --bin etna --features etna -- proptest SliceRefEmpty` |
| 018  | `slice_ref_empty_f330ef6_1` | quickcheck | `property_slice_ref_empty` | `witness_slice_ref_empty_case_nonempty_source` | `cargo run --release --bin etna --features etna -- quickcheck SliceRefEmpty` |
| 019  | `slice_ref_empty_f330ef6_1` | crabcheck | `property_slice_ref_empty` | `witness_slice_ref_empty_case_nonempty_source` | `cargo run --release --bin etna --features etna -- crabcheck SliceRefEmpty` |
| 020  | `slice_ref_empty_f330ef6_1` | hegel | `property_slice_ref_empty` | `witness_slice_ref_empty_case_nonempty_source` | `cargo run --release --bin etna --features etna -- hegel SliceRefEmpty` |

## Witness catalog

Each witness is a deterministic concrete test. Base build: passes. Variant-active build: fails.

- `witness_get_int_zero_nbytes_case_eight_zero_bytes` — buffer `[0u8; 8]`, `nbytes = 0` → `Buf::get_int(0) == 0` (no panic).
- `witness_get_int_sign_extension_case_four_ff_bytes` — buffer `[0xff; 4]`, `nbytes = 4` → `Buf::get_int(4) == -1`.
- `witness_chain_remaining_saturating_case_max_plus_one` — `FakeBuf(usize::MAX).chain(FakeBuf(1)).remaining() >= usize::MAX`.
- `witness_partialord_bytes_reversed_case_lhs_less` — lhs `[1,2,3]` vs rhs `Bytes::from([1,2,4])` → `Some(Less)`.
- `witness_slice_ref_empty_case_nonempty_source` — `Bytes::from(b"hello!").slice_ref(&[])` returns an empty `Bytes` without panicking.
