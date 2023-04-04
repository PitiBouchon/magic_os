#[allow(dead_code)]
pub fn sbi_print_str(s: &str) {
    for c in s.bytes() {
        if c.is_ascii() {
            sbi::legacy::console_putchar(c);
        }
    }
}

#[allow(dead_code)]
pub fn sbi_println_str(s: &str) {
    sbi_print_str(s);
    sbi::legacy::console_putchar(b'\n');
}

#[allow(dead_code)]
pub fn sbi_print_number_base10(num: usize) {
    sbi_print_number(num, 10, 0);
}

#[allow(dead_code)]
pub fn sbi_println_number_base10(num: usize) {
    sbi_print_number_base10(num);
    sbi::legacy::console_putchar(b'\n');
}

#[allow(dead_code)]
pub fn sbi_print_number_base2(num: u8) {
    sbi_print_number(num as usize, 2, 8);
}

#[allow(dead_code)]
pub fn sbi_print_number(num: usize, base: usize, min_number: u8) {
    let mut divisor = 1;
    let mut n = num;
    let mut i = 1;

    while n >= divisor * base || i < min_number {
        divisor *= base;
        i += 1;
    }

    while divisor > 0 {
        let v = (n / divisor) as u8;
        n %= divisor;
        divisor /= base;
        sbi_print_digit(v);
    }
}

fn sbi_print_digit(d: u8) {
    if d <= 9 {
        sbi::legacy::console_putchar(b'0' + d);
    }
}
