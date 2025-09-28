.section .rodata
.LC0:
    .string "the number is: %d\n"
.section .text
    .globl main
    .type main, @function
main:
    pushq %rbp
    movq %rsp, %rbp
    movl $1234, -4(%rbp)
    leaq .LC0(%rip), %rdi
    movq -4(%rbp), %rsi
    movl $0, %eax
    call printf
    leave
    ret
