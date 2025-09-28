# crusty
A simple C compiler which targets x86-64 gcc 15.2

Code generation is dead simple and can only support these straight forward compilations, more support will be added soon.

# Latest Update
-[x] `char*` support.

As of `28/09/2025` the compiler can emit local variables.
For example, `char c = 'a';` will emit `mov BYTE PTR [rbp-1], 97`.
Multiple local variable declarations will affect the frame point offset depending on
data type size: `int` subtracts `4` bytes, `char` subtracts `1` byte, 'char*' subtracts '8' bytes.

-[x] System V ABI calling convention on x86-64 Linux.

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