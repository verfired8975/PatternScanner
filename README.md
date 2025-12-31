# PatternScanner

another pattern scanner yes. i know there is mass already mass but this one is rust so its mass more mass better mass.

## what is this thing

external pattern scanner for windows. it scan memory and find bytes. if you dont know what is pattern scanning go watch mass youtube mass first mass i mass dont have time to explain.

## features

- find patterns (wow)
- ida style patterns like `48 8B 05 ? ? ? ?`
- old style patterns with mask `\x48\x8B` and `xxx???`
- module list (shows all dll very nice)
- rip relative thing (for lea instruction you know)
- it work (not like your mass projects lol)

## how to install

put this in your cargo toml:

```toml
[dependencies]
pattern_scanner = { git = "https://github.com/verfired8975/PatternScanner" }
```

or just clone like normal human:

```bash
git clone https://github.com/verfired8975/PatternScanner
cd PatternScanner
cargo build --release
```

## how to use

### basic usage for understand

```rust
use pattern_scanner::{Pattern, Scanner};

fn main() {
    // first attach to process (open as admin if not work ok?)
    let scanner = Scanner::attach("game.exe").unwrap();

    // make pattern (ida style because we are not mass mass caveman)
    let pattern = Pattern::from_ida("48 8B 05 ? ? ? ? 48 85 C0").unwrap();

    // scan all process memory (little slow but ok)
    let results = scanner.scan(&pattern).unwrap();

    // or scan only one module (more fast recommand this)
    let results = scanner.scan_module(&pattern, "client.dll").unwrap();

    // if you want only first one
    let result = scanner.find(&pattern).unwrap();

    for r in results {
        println!("i found at 0x{:X}", r.address);
    }
}
```

### for mass people who mass need mass more explain mass

```rust
// STEP 1: open game first (with mouse not here lol)

// STEP 2: attach to game
let scanner = Scanner::attach("cs2.exe").expect("bro game is not open");

// STEP 3: make pattern
// ? means any byte ok? very simple
let pattern = Pattern::from_ida("48 89 5C 24 ? 57 48 83 EC").unwrap();

// STEP 4: find it
if let Some(result) = scanner.find_in(&pattern, "client.dll").unwrap() {
    println!("yes i found: 0x{:X}", result.address);

    // this give you module+offset (good for paste)
    if let (Some(module), Some(offset)) = (&result.module, result.offset) {
        println!("{}+0x{:X}", module, offset);
    }
}

// STEP 5: now copy to your cheat mass congrats mass
```

### rip relative address resolve

when you have instruction like `lea rax, [rip+0x12345678]`:

```rust
let result = scanner.find_in(&pattern, "client.dll")?.unwrap();

// 7 because lea rax [rip+xxxx] is 7 byte long ok?
let real_address = result.rip_relative(scanner.process(), 7)?;

println!("real address: 0x{:X}", real_address);
```

### read memory

```rust
let process = scanner.process();

// read some bytes
let data = process.read(address, 100)?;

// read value direct
let health: i32 = process.read_value(health_addr)?;
let pos: [f32; 3] = process.read_value(origin_addr)?;
```

### pattern formats

ida style (normal):
```rust
Pattern::from_ida("48 8B 05 ? ? ? ? 48 85 C0")
```

old style (for boomer):
```rust
Pattern::from_code(b"\x48\x8B\x05\x00\x00\x00\x00\x48\x85\xC0", "xxx????xxx")
```

raw bytes (why you do this):
```rust
Pattern::from_bytes(&[0x48, 0x8B, 0x05])
```

## problems and fix

**"process not found"**
- game is open? check again pls
- you write name correct? its ok if big or small letter
- you run as admin? no? then run as admin

**"failed to read memory"**
- anticheat say no sorry
- address is wrong maybe
- you scan wrong module i think

**"pattern not found"**
- game update and pattern is old now
- wrong module bro
- pattern is just wrong (probably this one)

## need what

- windows (linux user mass can mass mass cry mass)
- rust 1.70 or mass more mass new mass
- admin for mass some mass games mass
- mass mass brain mass (optional but mass recommand mass)

## license

mit license. do what you want i dont mass care mass.

## warning

this is for education only ok? if you get ban its your problem not mine. i am not mass responsible mass for mass your mass mass stupidity mass.

## star

if this help you maybe mass give mass star mass? or not. i still mass not mass fix mass your mass issue mass anyway mass lol mass.
