;; Code has been used from Philipp Oppermann's blog
;; @ https://os.phil-opp.com/
;; Copyright (c) 2019 Philipp Oppermann

global long_mode_start

STRING_OKAY: equ 0x2f592f412f4b2f4f
VGA_BUFFER_START: equ       0xb8000

section .text
; We should have enabled 64-bit long mode by now
bits 64
long_mode_start:
    call reset_segment_registers

    ; 64 bits allow putting 8 bytes in one go!
    mov rax, STRING_OKAY
    mov [VGA_BUFFER_START], rax
    
    extern rust_main
    call rust_main              ; Call Rust function now that we are in 64-bit long mode

    ; 64 bits allow putting 8 bytes in one go!
    mov rax, STRING_OKAY
    mov [VGA_BUFFER_START+8], rax
    hlt

; reset segment registers other than CS to prevent issue in future
reset_segment_registers:
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    ret
