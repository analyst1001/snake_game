;; Code has been used from Philipp Oppermann's blog
;; @ https://os.phil-opp.com/
;; Copyright (c) 2019 Philipp Oppermann

section .multiboot_header
header_start:
    dd 0xe85250d6                   ; magic number for multiboot 2
    dd 0                            ; architecture = protected mode i386
    dd header_end - header_start    ; header_length
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))     ; change if any of the above is changed

    ; optional multiboot tags
    ; TODO: try out tag type 3

    ; end tag
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size
header_end:
