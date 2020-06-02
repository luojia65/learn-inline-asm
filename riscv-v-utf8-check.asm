.section .data
continuation_length:
    .quad 0x0101010101010101
    .quad 0x0403020200000000

.section .text
; a0: input address / has_error
; a1: input length
check_utf8:
    vsetvli t0, zero, e8 ; t0: vector length
    mv t1, 0xF4 ; t1: maximum UTF-8 allowed 0xF4
    mv t2, 0xED ; t2: all 0xED bytes
    mv t3, 0x9F ; t3: all 0x9F bytes
    mv t4, 0x8F ; t4: all 0x8F bytes
    la t5, continuation_length ; t5: mask of lengths
.loop: 
    sub a0, a0, t0 ; decrement pointer
    vle8.v v1, (a0) ; load input data v1
; check maximum UTF-8 allowed
    vmsltu.vx v2, v1, t1 ; check max into error v2
; calculate continuation length
    vsrl.vi v3, v1, 4 ; v3: input low nibble
    vlxei8.v v3, (t5), v3 ; v3: continuation length
; get current carries continuations

; check continuations

; get offset current bytes

; check first continuation max
    vmseq.vx v0, v1, t2 ; v0: 0xED byte mask 
    vmsltu.vx v4, v1, t3, v0.t ; v4: not less than 0x9F
    vmnor.mm v2, v2, v4 ; accumulate error
    vmseq.vx v0, v1, t1 ; v0: 0xF4 byte mask 
    vmsltu.vx v4, v1, t4, v0.t ; v4: not less than 0x8F
    vmnor.mm v2, v2, v4 ; accumulate error
; check overlong

; move current to prev

; next loop
    add a0, a0, t0 ; bump pointer
    bnez a1, loop ; next loop
; accumulate error
    vredor.vs v2, v2, v2
    vmv.x.s a0, v2
