.section .rodata
.LC0:
    .string "hello, world!\n"
.section .text
    .globl main
    .type main, @function
main:
    pushq %rbp
    movq %rsp, %rbp
    leaq .LC0(%rip), %rdi
    movl $0, %eax
    call printf
    leave
    ret
