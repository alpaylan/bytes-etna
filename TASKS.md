# bytes — ETNA Tasks

Total tasks: 20

## Task Index

| Task | Variant | Framework | Property | Witness |
|------|---------|-----------|----------|---------|
| 001 | `chain_remaining_saturating_2428c15_1` | proptest | `ChainRemainingSaturating` | `witness_chain_remaining_saturating_case_max_plus_one` |
| 002 | `chain_remaining_saturating_2428c15_1` | quickcheck | `ChainRemainingSaturating` | `witness_chain_remaining_saturating_case_max_plus_one` |
| 003 | `chain_remaining_saturating_2428c15_1` | crabcheck | `ChainRemainingSaturating` | `witness_chain_remaining_saturating_case_max_plus_one` |
| 004 | `chain_remaining_saturating_2428c15_1` | hegel | `ChainRemainingSaturating` | `witness_chain_remaining_saturating_case_max_plus_one` |
| 005 | `get_int_sign_extension_79fb853_1` | proptest | `GetIntSignExtension` | `witness_get_int_sign_extension_case_four_ff_bytes` |
| 006 | `get_int_sign_extension_79fb853_1` | quickcheck | `GetIntSignExtension` | `witness_get_int_sign_extension_case_four_ff_bytes` |
| 007 | `get_int_sign_extension_79fb853_1` | crabcheck | `GetIntSignExtension` | `witness_get_int_sign_extension_case_four_ff_bytes` |
| 008 | `get_int_sign_extension_79fb853_1` | hegel | `GetIntSignExtension` | `witness_get_int_sign_extension_case_four_ff_bytes` |
| 009 | `partialord_bytes_reversed_939a5ed_1` | proptest | `PartialordBytesReversed` | `witness_partialord_bytes_reversed_case_lhs_less` |
| 010 | `partialord_bytes_reversed_939a5ed_1` | quickcheck | `PartialordBytesReversed` | `witness_partialord_bytes_reversed_case_lhs_less` |
| 011 | `partialord_bytes_reversed_939a5ed_1` | crabcheck | `PartialordBytesReversed` | `witness_partialord_bytes_reversed_case_lhs_less` |
| 012 | `partialord_bytes_reversed_939a5ed_1` | hegel | `PartialordBytesReversed` | `witness_partialord_bytes_reversed_case_lhs_less` |
| 013 | `sign_extend_zero_nbytes_acd1e0f_1` | proptest | `GetIntZeroNbytes` | `witness_get_int_zero_nbytes_case_eight_zero_bytes` |
| 014 | `sign_extend_zero_nbytes_acd1e0f_1` | quickcheck | `GetIntZeroNbytes` | `witness_get_int_zero_nbytes_case_eight_zero_bytes` |
| 015 | `sign_extend_zero_nbytes_acd1e0f_1` | crabcheck | `GetIntZeroNbytes` | `witness_get_int_zero_nbytes_case_eight_zero_bytes` |
| 016 | `sign_extend_zero_nbytes_acd1e0f_1` | hegel | `GetIntZeroNbytes` | `witness_get_int_zero_nbytes_case_eight_zero_bytes` |
| 017 | `slice_ref_empty_f330ef6_1` | proptest | `SliceRefEmpty` | `witness_slice_ref_empty_case_nonempty_source` |
| 018 | `slice_ref_empty_f330ef6_1` | quickcheck | `SliceRefEmpty` | `witness_slice_ref_empty_case_nonempty_source` |
| 019 | `slice_ref_empty_f330ef6_1` | crabcheck | `SliceRefEmpty` | `witness_slice_ref_empty_case_nonempty_source` |
| 020 | `slice_ref_empty_f330ef6_1` | hegel | `SliceRefEmpty` | `witness_slice_ref_empty_case_nonempty_source` |

## Witness Catalog

- `witness_chain_remaining_saturating_case_max_plus_one` — base passes, variant fails
- `witness_get_int_sign_extension_case_four_ff_bytes` — base passes, variant fails
- `witness_partialord_bytes_reversed_case_lhs_less` — base passes, variant fails
- `witness_get_int_zero_nbytes_case_eight_zero_bytes` — base passes, variant fails
- `witness_slice_ref_empty_case_nonempty_source` — base passes, variant fails
