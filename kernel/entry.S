global _start

section .text
bits 32 ; tell NASM this is 32 bit code
_start:
    extern kmain
    call kmain
