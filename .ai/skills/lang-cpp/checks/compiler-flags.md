# Mandatory C++ Compiler Flags

## Clang & GCC Required Flags
```
-std=c++20
-Wall
-Wextra
-Wpedantic
-Wconversion
-Wsign-conversion
-Wshadow
-Wnon-virtual-dtor
-Wold-style-cast
-Wcast-align
-Wunused
-Woverloaded-virtual
-Wnull-dereference
-Wdouble-promotion
-Wformat=2
-Werror
```

## Sanitizer Flags for Verification Target
```
-fsanitize=address,undefined,leak
-fno-omit-frame-pointer
-fno-optimize-sibling-calls
```
