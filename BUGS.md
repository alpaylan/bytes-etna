# bytes — Injected Bugs

Total mutations: 5

## Bug Index

| # | Name | Variant | File | Injection | Fix Commit |
|---|------|---------|------|-----------|------------|
| 1 | `sign_extend_zero_nbytes` | `sign_extend_zero_nbytes_acd1e0f_1` | `src/buf/buf_impl.rs:86` | `patch` | `acd1e0ffb8f076225759b8005d04f65ef77cccca` |
| 2 | `get_int_sign_extension` | `get_int_sign_extension_79fb853_1` | `src/buf/buf_impl.rs:949` | `patch` | `79fb85323cf4cf14d9b85f487b65fc147030cf4b` |
| 3 | `chain_remaining_saturating` | `chain_remaining_saturating_2428c15_1` | `src/buf/chain.rs:133` | `patch` | `2428c152a67c06057a98d9d29b08389cb3429c1f` |
| 4 | `partialord_bytes_reversed` | `partialord_bytes_reversed_939a5ed_1` | `src/bytes.rs:791` | `patch` | `939a5edf3d28acfc390eedc46eab69843ba363b6` |
| 5 | `slice_ref_empty` | `slice_ref_empty_f330ef6_1` | `src/bytes.rs:407` | `patch` | `f330ef6c4d806abecadca945f682436737399039` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `sign_extend_zero_nbytes_acd1e0f_1` | `property_get_int_zero_nbytes` | `witness_get_int_zero_nbytes_case_eight_zero_bytes` |
| `get_int_sign_extension_79fb853_1` | `property_get_int_sign_extension` | `witness_get_int_sign_extension_case_four_ff_bytes` |
| `chain_remaining_saturating_2428c15_1` | `property_chain_remaining_saturating` | `witness_chain_remaining_saturating_case_max_plus_one` |
| `partialord_bytes_reversed_939a5ed_1` | `property_partialord_bytes_reversed` | `witness_partialord_bytes_reversed_case_lhs_less` |
| `slice_ref_empty_f330ef6_1` | `property_slice_ref_empty` | `witness_slice_ref_empty_case_nonempty_source` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `property_get_int_zero_nbytes` | ✓ | ✓ | ✓ | ✓ |
| `property_get_int_sign_extension` | ✓ | ✓ | ✓ | ✓ |
| `property_chain_remaining_saturating` | ✓ | ✓ | ✓ | ✓ |
| `property_partialord_bytes_reversed` | ✓ | ✓ | ✓ | ✓ |
| `property_slice_ref_empty` | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. sign_extend_zero_nbytes
- **Variant**: `sign_extend_zero_nbytes_acd1e0f_1`
- **Location**: `src/buf/buf_impl.rs:86` (via `patches/sign_extend_zero_nbytes_acd1e0f_1.patch`)
- **Property**: `property_get_int_zero_nbytes`
- **Witness(es)**: `witness_get_int_zero_nbytes_case_eight_zero_bytes`
- **Fix commit**: `acd1e0ffb8f076225759b8005d04f65ef77cccca` — `Fix get_int if nbytes is zero (#806)`
- **Invariant violated**: `Buf::get_int(0)` and `Buf::get_int_le(0)` must return `0` without panicking.
- **How the mutation triggers**: the patch removes the `if nbytes == 0 { 0 }` guard in `sign_extend`, so a zero-byte call attempts `val << 64`, which panics with `attempt to shift left with overflow`.

### 2. get_int_sign_extension
- **Variant**: `get_int_sign_extension_79fb853_1`
- **Location**: `src/buf/buf_impl.rs:949` (via `patches/get_int_sign_extension_79fb853_1.patch`)
- **Property**: `property_get_int_sign_extension`
- **Witness(es)**: `witness_get_int_sign_extension_case_four_ff_bytes`
- **Fix commit**: `79fb85323cf4cf14d9b85f487b65fc147030cf4b` — `fix: apply sign extension when decoding int (#732)`
- **Invariant violated**: `Buf::get_int(n)` on a buffer of `n` bytes of `0xff` must equal `-1` (two's-complement sign extension to `i64`).
- **How the mutation triggers**: the patch rewires `get_int`/`get_int_le` to use `buf_get_impl!(be => …, i64, nbytes)` in place of `sign_extend(self.get_uint(nbytes), nbytes)`, so the high bit of a partial integer is dropped and `[0xff; 4]` returns `4294967295` instead of `-1`.

### 3. chain_remaining_saturating
- **Variant**: `chain_remaining_saturating_2428c15_1`
- **Location**: `src/buf/chain.rs:133` (via `patches/chain_remaining_saturating_2428c15_1.patch`)
- **Property**: `property_chain_remaining_saturating`
- **Witness(es)**: `witness_chain_remaining_saturating_case_max_plus_one`
- **Fix commit**: `2428c152a67c06057a98d9d29b08389cb3429c1f` — `Panic on integer overflow in Chain::remaining (#482)`
- **Invariant violated**: `Chain::remaining()` must never silently wrap; when the two halves together exceed `usize::MAX` the result must be at least `max(a, b)` (saturating-add behaviour).
- **How the mutation triggers**: `saturating_add` is replaced by `wrapping_add`, so `Chain(usize::MAX, 1).remaining()` returns `0`, violating the saturating lower bound.

### 4. partialord_bytes_reversed
- **Variant**: `partialord_bytes_reversed_939a5ed_1`
- **Location**: `src/bytes.rs:791` (via `patches/partialord_bytes_reversed_939a5ed_1.patch`)
- **Property**: `property_partialord_bytes_reversed`
- **Witness(es)**: `witness_partialord_bytes_reversed_case_lhs_less`
- **Fix commit**: `939a5edf3d28acfc390eedc46eab69843ba363b6` — `Fix reversed arguments in PartialOrd impls (#358)`
- **Invariant violated**: `<[u8] as PartialOrd<Bytes>>::partial_cmp(lhs, rhs)` must equal `<[u8]>::partial_cmp(lhs, rhs.as_ref())`.
- **How the mutation triggers**: the body is swapped to `other.partial_cmp(self)`, inverting the comparison. For `lhs=[1,2,3]` and `rhs=[1,2,4]` the buggy impl returns `Some(Greater)` instead of `Some(Less)`.

### 5. slice_ref_empty
- **Variant**: `slice_ref_empty_f330ef6_1`
- **Location**: `src/bytes.rs:407` (via `patches/slice_ref_empty_f330ef6_1.patch`)
- **Property**: `property_slice_ref_empty`
- **Witness(es)**: `witness_slice_ref_empty_case_nonempty_source`
- **Fix commit**: `f330ef6c4d806abecadca945f682436737399039` — `Do not panic on Bytes::slice_ref on empty slice (#355)`
- **Invariant violated**: `Bytes::slice_ref(&[])` must return an empty `Bytes` for any source `Bytes`, including non-empty ones, without panicking.
- **How the mutation triggers**: the patch removes the `if subset.is_empty() { return Bytes::new(); }` early-return, so the range-check hits `subset.as_ptr() < self.as_ptr()` and panics with `subset slice is not a subset of self`.
