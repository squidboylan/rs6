global _start

section .text
bits 32 ; tell NASM this is 16 bit code
_start:
    extern rust_main
    call rust_main
