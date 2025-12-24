# Research - Engine V2 Matrix System

**Status**: Final Design Complete
**Date**: 2025-12-24

---

## Files

| File | Description |
|------|-------------|
| `matrix-system.md` | **Complete matrix design** - U1-U7, M1-M8, E1-E5 tables |

## Memory Summary

| Category | Size |
|----------|------|
| Input Processing (U1-U7) | 141 bytes |
| Vietnamese Validation (M1-M8) | ~950 bytes |
| English Validation (E1-E5) | ~2KB |
| **Total** | **~3.1KB** |

## Design Principles

1. **Zero if-else in hot path** - Every decision = matrix lookup
2. **Single lookup per step** - No chained conditions
3. **Packed data** - Bit flags instead of separate tables
4. **5 states** - EMPTY, INIT, VOW, DIA, FIN

## Key Improvements (v1 → v2)

- 87% memory reduction (input processing)
- Eliminated case-by-case logic
- Single byte encodes action + state
- Unified letter classification

## Archive

Old v1 designs and intermediate reports in `archived/` folder.
