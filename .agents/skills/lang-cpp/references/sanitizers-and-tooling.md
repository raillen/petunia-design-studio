# C++ Sanitizers & Verification Tooling Reference

## 1. AddressSanitizer (ASan)
- Flags: `-fsanitize=address -fno-omit-frame-pointer`
- Catches: Out-of-bounds heap/stack/global accesses, Use-After-Free (UAF), Use-After-Return, Double-Free.

## 2. UndefinedBehaviorSanitizer (UBSan)
- Flags: `-fsanitize=undefined`
- Catches: Signed integer overflow, null pointer dereference, misaligned memory access, Vptr violations, invalid enum casts.

## 3. ThreadSanitizer (TSan)
- Flags: `-fsanitize=thread`
- Catches: Data races between concurrent threads, deadlocks, incorrect mutex lock orders. Note: Mutually exclusive with ASan.

## 4. LeakSanitizer (LSan)
- Flags: `-fsanitize=leak`
- Catches: Memory leaks upon process exit.
