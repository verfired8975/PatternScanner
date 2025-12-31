use pattern_scanner::{Pattern, Scanner};

fn main() {
    let scanner = match Scanner::attach("notepad.exe") {
        Ok(s) => s,
        Err(e) => {
            println!("olmadi: {}", e);
            return;
        }
    };

    println!("process acildi, pid: {}", scanner.process().pid());

    let modules = scanner.process().modules().unwrap();
    for m in &modules {
        println!("  {} @ 0x{:X} ({}KB)", m.name, m.base, m.size / 1024);
    }

    // ida style pattern
    let pattern = Pattern::from_ida("48 89 5C 24 ? 48 89 74 24").unwrap();

    println!("\ntum process taraniyor...");
    match scanner.scan(&pattern) {
        Ok(results) => {
            println!("{} tane buldum", results.len());
            for r in results.iter().take(10) {
                match (&r.module, r.offset) {
                    (Some(m), Some(off)) => println!("  {}+0x{:X}", m, off),
                    _ => println!("  0x{:X}", r.address),
                }
            }
        }
        Err(e) => println!("hata: {}", e),
    }

    // tek modülde ara
    if let Some(main_mod) = modules.first() {
        println!("\nsadece {} icinde ariyorum...", main_mod.name);
        if let Ok(Some(result)) = scanner.find_in(&pattern, &main_mod.name) {
            println!("buldum: 0x{:X}", result.address);
        }
    }
}
