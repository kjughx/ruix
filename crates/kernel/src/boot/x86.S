.code16
.org 0x7c00
.section .boot, "ax"

.global _entry

.intel_syntax
_entry:
    jmp short step1
    nop
.att_syntax

OEMIdentifier:     .byte 'R', 'U', 'I', 'X', ' ', ' ', ' ', ' '
BytesPerSector:    .word 0x200
SectorsPerCluster: .byte 0x80
ReservedSectors:   .word 200
FATCopies:         .byte 0x02
RootDirEntries:    .word 0x40
NumSectors:        .word 0x00
MediaType:         .byte 0xF8
SectorsPerFat:     .word 0x100
SectorsPerTrack:   .word 0x20
NumberOfHeads:     .word 0x40
HiddenSectors:     .long 0x00
SectorsBig:        .long 0x773594

// Extended BPB (Dos 4.0)
DriveNumber:       .byte 0x80
WinNTBit:          .byte 0x00
Signature:         .byte 0x29
VolumeID:          .long 0xD105
VolumeIDString:    .byte 'R', 'U', 'I', 'X', ' ', ' ', ' ', 'B', 'O', 'O', 'T'
SystemIDString:    .byte 'F', 'A', 'T', '1', '6', ' ', ' ', ' '

step1:
    jmp $0, $step2

step2:
    cli

    mov $0x00, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss
    mov %ax, %fs
    mov %ax, %gs

    mov $0x7c00, %sp

.load_protected:
    lgdt gdt_descriptor
    mov %cr0, %eax
    or $0x1, %eax
    mov %eax, %cr0

    jmp $0x08, $load32

gdt_start:
gdt_null:
    .long 0x0
    .long 0x0

// offset 0x8
gdt_code:           // CS should point here
    .word 0xffff    // segment limit first 0-15 bits
    .word 0         // Base first 0-15 bits
    .byte 0         // Base first 16-23 bits
    .byte 0x9a      // Access byte
    .byte 0b11001111 // High 4 bit flags and low 4 bit flags
    .byte 0         // Base 24-31 bits

// offset 0x10
gdt_data:        // DS, SS, ES, FS, GS should point here
    .word 0xffff    // segment limit first 0-15 bits
    .word 0         // Base first 0-15 bits
    .byte 0         // Base first 16-23 bits
    .byte 0x92      // Access byte
    .byte 0b11001111 // High 4 bit flags and low 4 bit flags
    .byte 0         // Base 24-31 bits

gdt_end:
gdt_descriptor:
    .word gdt_end - gdt_start -1
    .long gdt_start

.code32
load32:
    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss
    mov %ax, %fs
    mov %ax, %gs

    mov $1, %eax
    mov $250, %ecx
    mov $0x100000, %edi
    call ata_lba_read
    ljmp $0x08, $0x100000

ata_lba_read:
    mov %eax, %ebx //Backup the LBA
    //Send the highest 8 bits of the lba to hard disk controller
    .intel_syntax
    shr eax, 24
    .att_syntax
    or $0xe0, %eax //Select master drive
    mov $0x1F6, %dx
    out %al, %dx
    //Finished sending the highest 8 bits of the lba

    //Send total #sectors to read
    mov %ecx, %eax
    mov $0x1F2, %dx
    out %al, %dx
    //Finished sending #sectors

    //Send more bits of the LBA
    mov %ebx, %eax //Restore backed up LBA
    mov $0x1F3, %dx
    out %al, %dx
    //Finished sending more bits

    //Send more bits of LBA
    mov %ebx, %eax //Restore backed up LBA
    mov $0x1F4, %dx
    .intel_syntax
    shr eax, 8
    .att_syntax
    out %al, %dx
    //Finished

    //Send upper 16 bits of LBA
    mov $0x1F5, %dx
    mov %ebx, %eax //Restore backed up LBA
    .intel_syntax
    shr eax, 16
    .att_syntax
    out %al, %dx
    //Finished

    mov $0x1F7, %dx
    mov $0x20, %al
    out %al, %dx

    //Read all sectors into memory
.next_sector:
    push %ecx

//Check if we can read
.try_again:
    mov $0x1F7, %dx
    in %dx, %al
    test $8, %al
    jz .try_again

    //We need to read 256 words at a time
    mov $256, %ecx
    mov $0x1F0, %dx
    rep insw //Read a word from dx into edi, ecx times
    pop %ecx
    loop .next_sector
    //Finished reading sectors into memory

    ret

.fill 510 - (.- _entry), 1, 0

.word 0xAA55
