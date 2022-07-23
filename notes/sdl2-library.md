# SDL 2.0 development libraries

## SDL 2.0 library setup

official setup guide: [Rust-SDL2](https://github.com/Rust-SDL2/rust-sdl2#sdl20-development-libraries)

### my quick and dirty solution

1. download development library: [SDL2-devel-2.0.22-VC.zip](https://www.libsdl.org/release/SDL2-devel-2.0.22-VC.zip)
2. unpack zip archive
3. navigate to `SDL2-devel-2.0.22-VC\SDL2-2.0.22\lib\x64`
4. copy and paste all files into the project root folder
5. add the crate to the `Cargo.toml` file:

```toml
# ...

[dependencies.sdl2]
version = "0.35"
default-features = true
```

6. build project using `cargo build`

## ship project

1. run `cargo build`
2. extract `.exe` file and put it into a folder
3. add the `SDL2.ddl` file
4. run the `.exe` file