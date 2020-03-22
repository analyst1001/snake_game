global start
extern long_mode_start

VGA_BUFFER_START: equ       0xb8000
STRING_OK: equ              0x2f4b2f4f
STRING_ER: equ              0x4f524f45
STRING_RCOLON: equ          0x4f3a4f52
STRING_DOUBLE_SPACE: equ    0x4f204f20

NOT_MULTIBOOT_COMPLIANT_BOOTLOADER_ERR: equ 0x30
CPUID_NOT_SUPPORTED_ERR: equ                0x31
CANNOT_USE_LONG_MODE_ERR: equ               0x32

section .text
; hopefully, the bootloader has put us in 32 bit protected mode
bits 32
start:
    ; use the stack reserved below
    mov esp, stack_top
    
    ; check multiboot bootloader
    call check_multiboot_compliant_bootloader

    ; check CPUID supported
    call check_cpuid_supported
    
    ; check if we can use 64 bit long mode
    call check_long_mode

    ; set up Identity paging
    call set_up_page_tables

    ; enable paging and enter the long mode
    call enable_paging_and_long_mode

    ; load 64 bit GDT
    lgdt [gdt64.pointer]

    ; far jump to load the code segment register
    jmp gdt64.code:long_mode_start
    
    ; halt further processing
    ;;hlt

; print `ERR: ` and the given error code on VGA buffer
; parameter: error code in ascii in al register
print_error:
    ; print "ERR: "
    mov dword [VGA_BUFFER_START], STRING_ER
    mov dword [VGA_BUFFER_START + 0x4], STRING_RCOLON
    mov dword [VGA_BUFFER_START + 0x8], STRING_DOUBLE_SPACE
    ; overwrite last space character
    mov byte  [VGA_BUFFER_START + 0xa], al
    ; halt further processing
    hlt

; check if the binary was loaded by a multiboot compliant bootloader
check_multiboot_compliant_bootloader:
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret
.no_multiboot:
    mov al, NOT_MULTIBOOT_COMPLIANT_BOOTLOADER_ERR
    jmp print_error

; check if CPUID is supported
check_cpuid_supported:
    
    ; push FLAGS register value on stack
    pushfd
    ; load FLAGS register value in EAX
    pop eax

    ; Make a copy for comparison later
    mov ecx, eax

    ; flip 'ID' bit
    xor eax, 0x200000

    ; push EAX to stack with inverted bit value
    push eax
    ; try to load inverted bit value in FLAGS register
    popfd

    ; push the FLAGS register value that was loaded after bit flipping
    pushfd
    ; pop the FLAGS register value that was loaded after bit flipping in EAX
    pop eax

    ; If the bit was flipped, it needs to be restored. Restore it using copy present in ECX
    push ecx
    popfd

    ; Check if the flipped value was loaded in FLAGS register with bit flipped (modification persisted?).
    cmp ecx, eax
    je .no_cpuid
    ret
.no_cpuid:
    ; print error if CPUID is not supported
    mov al, CPUID_NOT_SUPPORTED_ERR
    jmp print_error

; check if we can use 64-bit long mode
check_long_mode:
    ; check if extended functions of CPUID are available
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    ; extended functions not available. Long mode can't be used
    jb .no_long_mode

    ; test if long mode can be used using extended CPUID functions
    mov eax, 0x80000001
    cpuid
    ; test lm bit in EDX
    test edx, 1<<29
    ; if lm bit is 0, long mode is not present
    jz .no_long_mode
    ret
.no_long_mode:
    mov al, CANNOT_USE_LONG_MODE_ERR
    jmp print_error

; set up page tables for identity paging in long mode
set_up_page_tables:
    mov eax, p3_table
    ; set present + writable flags for p3_table entry
    or eax, 0b11
    ; insert entry for p3_table in p4_table
    mov [p4_table], eax

    mov eax, p2_table
    ; set present + writable flags for p2_table entry
    or eax, 0b11
    ; insert entry for p2_table in p3_table
    mov [p3_table], eax

    ; ECX works a the counter
    mov ecx, 0

; map first 1 GiB virtual memory to first 1 GiB physical memory (512 hugepages of size 2MiB)
.map_p2_table:
    ; calculate starting offset of hugepage
    mov eax, 0x200000
    mul ecx
    ; set appropriate flags for the hugepage
    or eax, 0b10000011
    ; insert hugepage entry in p2_table at appropriate offset
    mov [p2_table + ecx*8], eax

    ; increment counter variable
    inc ecx
    ; check if all 512 entries are filled
    cmp ecx, 512
    jb .map_p2_table

    ret

; enable paging and long mode
enable_paging_and_long_mode:
    ; load starting address of p4_table to CR3 register
    mov eax, p4_table
    mov cr3, eax

    ; first enable PAE
    mov eax, cr4
    or eax, 1<<5
    mov cr4, eax

    ; set up LME bit by writing to EFER MSR
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1<<8
    wrmsr

    ; now enable paging via PG bit in CR0 register
    mov eax, cr0
    or eax, 1<<31
    mov cr0, eax

    ret

section .rodata:
; GDT for 64 bit long mode
gdt64:
    dq 0        ; zero entry
.code: equ $ - gdt64    ; offset of segment descriptor entry of code segment from beginning of GDT
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53)    ; code segment (executable + descriptor type for code segment + present + 64-bit)
; pointer to lgdt for loading the GDT
.pointer:
    ; length of GDT
    dw $ - gdt64 - 1
    ; pointer to starting of GDT 
    dq gdt64

section .bss
; align to keep last 12 bits available for flags
align 4096

; reserve space for page tables
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
; we don't need p1_table because we reserved 2MiB hugepages

; TODO: Introduce guard page. Absence of it causes Page tables to be overwritten, causing "interesting" issues
; reserve space for stack
stack_bottom:
    resb 4096 * 4
; stack grows from higher address to lower address
stack_top:
