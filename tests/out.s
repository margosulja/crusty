.section .text
    .globl main
    .type main, @function
main:
    pushq %rbp
    movq %rsp, %rbp
    movl $42, -4(%rbp)
    movl $0, %eax
    popq %rbp
    ret
