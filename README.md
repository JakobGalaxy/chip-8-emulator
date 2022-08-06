# CHIP-8 emulator

This is a small learning project to get familiar with emulator development, the [Rust](https://www.rust-lang.org/)
programming language and the [SDL2](https://www.libsdl.org/index.php) library.

> **Disclaimer:** As I am still learning Rust, my code is probably pretty terrible and full of bugs ... so you are
> forewarned.

## setup

If you want to try the emulator yourself, just follow the steps below:

1. download the latest version [here](https://github.com/JakobGalaxy/chip-8-emulator/releases)
2. unpack the archive
3. run `chip-8-emulator.exe`
4. when asked if you want to continue with the default configuration, input `y` for yes
5. done!

Of course, my welcome program is pretty boring so just grab some programs from the internet and have fun. I've linked to
a few programs that I've tried myself [below](#tested-programs). Before you can actually run other programs you need to
change the `program_path` attribute in the config file (`./config/chip8-emulator.toml`) either via the menu or by
editing the file directly.

If you want to use other fonts, (again) just grab some from the internet. As with the program, you need to change
the `font_path` attribute in the config. I've linked some fonts [below](#fonts) that I could find.

> **Note:** The fonts are often provided as text files. In order for the emulator to be able to interpret them, you need
> to convert to binary first. The result should be an 80 byte (1 byte per row * 5 rows per char * 16 chars) long binary
> file.

## keypad

The original keypad was organized like this:

```
1 2 3 C
4 5 6 D
7 8 9 E
A 0 B F
```

I've mapped these keys to the following:

```
1 2 3 4
Q W E R
A S D F
Z X C V
```

You can also use `Y` instead of `Z`.

## useful resources

Thanks to all the authors of these resources for their great effort!

- [Guide to making a CHIP-8 emulator - Tobias V. Langhoff](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/)
- [CHIP-8 emulator in Rust. Part 1 - Dhole's blog](https://dhole.github.io/post/chip8_emu_1/)
- [Simple DirectMedia Layer - Wikipedia](https://en.wikipedia.org/wiki/Simple_DirectMedia_Layer)
- [CHIP-8 - Wikipedia](https://en.wikipedia.org/wiki/CHIP-8)

## fonts

Thanks to [zZeck](https://github.com/zZeck) for collecting all of these fonts!

- [chip48](https://github.com/mattmikolay/chip-8/files/3365168/chip48font.txt)
- [cosmacvip](https://github.com/mattmikolay/chip-8/files/3365169/cosmacvipfont.txt)
- [dream6800](https://github.com/mattmikolay/chip-8/files/3365170/dream6800font.txt)
- [eti660](https://github.com/mattmikolay/chip-8/files/3365171/eti660font.txt)

## tested programs

Thanks to all the people who wrote these programs!

### tests

- [opcode test](https://github.com/corax89/chip8-test-rom/blob/master/test_opcode.ch8)
- [delay timer test](https://github.com/mattmikolay/chip-8/blob/master/delaytimer/delay_timer_test.ch8)
- [random number test](https://github.com/mattmikolay/chip-8/blob/master/randomnumber/random_number_test.ch8)

### demos

- [maze](https://github.com/cj1128/chip8-emulator/blob/master/rom/Maze)
- [heart monitor](https://github.com/mattmikolay/chip-8/blob/master/heartmonitor/heart_monitor.ch8)
- [morse code](https://github.com/mattmikolay/chip-8/blob/master/morsecode/morse_demo.ch8)

### games

- [pong](https://github.com/cj1128/chip8-emulator/blob/master/rom/PONG)
- [chipquarium](https://github.com/mattmikolay/chip-8/blob/master/chipquarium/chipquarium.ch8)

> but can it run crysis?

## license

Source code in this repository is licensed under the MIT license. For more information, see the included [LICENSE](https://github.com/JakobGalaxy/chip-8-emulator/blob/main/LICENSE) file.