.LC0: .string "hello world!"
main:
    push rbp
    mov rbp, rsp
    mov QWORD PTR [rbp-8], OFFSET FLAT:.LC0
    mov eax, 0
    pop rbp
    ret
