bits 16 ; tell NASM this is 16 bit code
global start
start:
  ; disable interrupts, who knows what the BIOS did
  cli
  ; clear ax, ds (data segment), es (extra segment), ss (stack segment)
  xor    ax, ax
  mov    ds, ax
  mov    es, ax
  mov    ss, ax

  ; enable A20
seta20.1:
  in     al,0x64               ; Wait for not busy
  test   al,0x2
  jnz    seta20.1

  mov    al,0xd1               ; 0xd1 -> port 0x64
  out    0x64,al
seta20.2:
  in     al,0x64               ; Wait for not busy
  test   al,0x2
  jnz    seta20.2
  mov    al,0xdf               ; 0xdf -> port 0x60
  out    0x60,al

  ; setup a temp gdt
setgdt:
  lgdt   [gdt_pointer]

  ; enter protected mode! (not 32 bit yet)
  mov   eax, cr0
  or    eax, 1    ; Set the protected mode bit
  mov   cr0, eax

  ; ljmp into 32 bit mode!
  jmp  8:start32 ; this sets cs to 0x1000 (where rsv6 is loaded)

bits 32  ; we 32 bit now
start32:
  ; setup segmentation registers, im copying how xv6 does it
  ; we'll have to change this based on where we put certain things in linking,
  ; etc
  ; Set up the protected-mode data segment registers
  mov    ax, 0x10
  mov    ds, ax
  mov    es, ax
  mov    ss, ax
  mov    ax, 0
  mov    fs, ax                ; set fs to 0
  mov    gs, ax                ; set fs to 0

  mov    esp, start  ; set the stack pointer to be before the bootloader,
                     ; this grows down
  extern bootmain
  call   bootmain

align 4
gdt_null:
  ; all zeroes, that's just how this is
  dq 0x0
gdt_code:
  ; segment limit
  dw 0xffff
  ; segment base
  dw 0x0
  ; base high bits
  db 0x0
  ; access flags, sets rw, executable, ring 0, and present bits
  db 10011010b
  ; flags and 4 limit high bits (low bits of the byte)
  ; flags set granularity to 4KiB (limit is in 4KiB rather than B)
  db 11001111b
  ; more base high bits
  db 0x0
gdt_data:
  dw 0xffff
  dw 0x0
  ; base high bits
  db 0x0
  ; access flags, sets rw, ring 0, and present bits
  db 10010010b
  ; flags and 4 limit high bits (low bits of the byte)
  ; flags set granularity to 4KiB (limit is in 4KiB rather than B)
  db 11001111b
  ; more base high bits
  db 0x0

gdt_pointer:
    dw 0x17 ; gdt is 24B, then we subtract 1
    dd gdt_null ; gdt start
