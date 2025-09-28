# crusty
A simple C compiler which targets x86-64 gcc 15.2

Code generation is dead simple and can only support these straight forward compilations, more support will be added soon.

# Features & Info
• Local variable declarations.

• Function declarations.

• `char*`, `char`, `int` support.

• System V ABI calling convention on x86-64 Linux.

```c
int main() {
    int x = 10;
    char w = 'a';
}
```

```asm
main:
    push rbp
    mov rbp, rsp
    
    ; int x = 10
    mov DWORD PTR [rbp-4], 10
    ; char w = 'a';
    mov BYTE PTR [rbp-5], 97
    
    mov eax, 0
    pop rbp
    ret
```

#### Emission
```asm
main:
    ; function prologue
    push rbp
    mov rbp, rsp
    ; body
    mov DWORD PTR [rbp-4], 5
    ; function epilogue
    mov eax, 0
    pop rbp
    ret
```

`0` is implicitly returned from the function if a return is not available.