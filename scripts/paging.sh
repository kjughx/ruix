#!/bin/sh


# (gdb)
set $pte_addr = $cr3 + ((0x400000 >> 12) & 0x3FF) * 4

set $pte = *($pte_addr as *const u32)
set $pte = *(unsigned int*)$pte_addr
# Present (P) bit
print /t $pte & 0x1
# Write (W) bit
print /t ($pte >> 1) & 0x1
# User (U) bit
print /t ($pte >> 2) & 0x1
print /t ($pte >> 3) & 0x1  # Page-level write-through (PWT) bit
print /t ($pte >> 4) & 0x1  # Page-level cache disable (PCD) bit
print /t ($pte >> 5) & 0x1  # Accessed (A) bit
print /t ($pte >> 6) & 0x1  # Dirty (D) bit
print /t ($pte >> 7) & 0x1  # Page size (PS) bit
print /t ($pte >> 8) & 0x1  # Global (G) bit
