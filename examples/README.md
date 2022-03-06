Suggested by Guillaume Gomez, it would be great to refactor the following Rust code 
```rust
let mut i: libc::c_int = 1 as libc::c_int;
while i < n {
}
```
into:
```rust
for i in 1..n {
}
```
