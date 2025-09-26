# crusty
A simple C compiler which targets x86-64 gcc 15.2

Code generation is dead simple and can only support these straight forward compilations, more support will be added soon.

# Example
```c
int main() {
    int x = 5;
}
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