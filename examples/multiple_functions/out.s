.section .rodata
.LC0:
    .string "hello %s!\n"
.LC1:
    .string "margo"
.section .text
    .globl main
    .type main, @function
greet:
    pushq %rbp
    movq %rsp, %rbp
    subq $16, %rsp
    movq %rdi, -8(%rbp)
    leaq .LC0(%rip), %rdi
    movq -8(%rbp), %rsi
    movl $0, %eax
    call printf
    leave
    ret
main:
    pushq %rbp
    movq %rsp, %rbp
    leaq .LC1(%rip), %rdi
    call greet
    leave
    ret
