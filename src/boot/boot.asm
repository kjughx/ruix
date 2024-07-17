ORG 0x7c00
BITS 16

CODE_SEG equ gdt_code - gdt_start
DATA_SEG equ gdt_data - gdt_start

jmp short step1
nop

; FAT16 Header
OEMIdentifier     db 'PHIX    ' ; 8 bytes
BytesPerSector    dw 0x200      ; 512 Bytes per sector
SectorsPerCluster db 0x80       ; 128 Sectors
ReservedSectors   dw 200        ; Reserved for kernel (200 * 512 = 100KB)
FATCopies         db 0x02       ; Two copies (original and backup)
RootDirEntries    dw 0x40       ;
NumSectors        dw 0x00
MediaType         db 0xF8
SectorsPerFat     dw 0x100
SectorsPerTrack   dw 0x20
NumberOfHeads     dw 0x40
HiddenSectors     dd 0x00
SectorsBig        dd 0x773594

; Extended BPB (Dos 4.0)
DriveNumber       db 0x80
WinNTBit          db 0x00
Signature         db 0x29
VolumeID          dd 0xD105
VolumeIDString    db 'PHIX   BOOT' ; 11 Bytes
SystemIDString    db 'FAT16   '

step1:
    jmp 0:step2

step2:
    cli ; Disable interrupts
    mov ax, 0x00
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7c00

.load_protected:
    lgdt[gdt_descriptor]
    mov eax, cr0
    or eax, 0x1
    mov cr0, eax
    jmp CODE_SEG:load32

gdt_start:
gdt_null:
    dd 0x0
    dd 0x0

; offset 0x8
gdt_code:        ; CS should point here
    dw 0xffff    ; segment limit first 0-15 bits
    dw 0         ; Base first 0-15 bits
    db 0         ; Base first 16-23 bits
    db 0x9a      ; Access byte
    db 11001111b ; High 4 bit flags and low 4 bit flags
    db 0         ; Base 24-31 bits

; offset 0x10
gdt_data:        ; DS, SS, ES, FS, GS should point here
    dw 0xffff    ; segment limit first 0-15 bits
    dw 0         ; Base first 0-15 bits
    db 0         ; Base first 16-23 bits
    db 0x92      ; Access byte
    db 11001111b ; High 4 bit flags and low 4 bit flags
    db 0         ; Base 24-31 bits

gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start -1
    dd gdt_start

[BITS 32]
load32:
    mov eax, 1
    mov ecx, 100
    mov edi, 0x0100000
    call ata_lba_read
    jmp CODE_SEG:0x0100000

ata_lba_read:
    mov ebx, eax ; Backup the LBA
    ; Send the highest 8 bits of the lba to hard disk controller
    shr eax, 24
    or eax, 0xe0 ; Select master drive
    mov dx, 0x1F6
    out dx, al
    ; Finished sending the highest 8 bits of the lba

    ; Send total #sectors to read
    mov eax, ecx
    mov dx, 0x1F2
    out dx,al
    ; Finished sending #sectors

    ; Send more bits of the LBA
    mov eax, ebx ; Restore backed up LBA
    mov dx, 0x1F3
    out dx, al
    ; Finished sending more bits

    ; Send more bits of LBA
    mov eax, ebx ; Restore backed up LBA
    mov dx, 0x1F4
    shr eax, 8
    out dx, al
    ; Finished

    ; Send upper 16 bits of LBA
    mov dx, 0x1F5
    mov eax, ebx ; Restore backed up LBA
    shr eax, 16
    out dx, al
    ; Finished

    mov dx, 0x1F7
    mov al, 0x20
    out dx, al

    ; Read all sectors into memory
.next_sector:
    push ecx

; Check if we can read
.try_again:
    mov dx, 0x1F7
    in al, dx
    test al, 8
    jz .try_again

    ; We need to read 256 words at a time
    mov ecx, 256
    mov dx, 0x1F0
    rep insw ; Read a word from dx into edi, ecx times
    pop ecx
    loop .next_sector
    ; Finished reading sectors into memory

    ret

times 510-($ - $$) db 0
dw 0xAA55
