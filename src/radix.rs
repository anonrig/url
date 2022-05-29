pub struct Radix {
    x: i32,
    radix: u32,
}

impl Radix {
    pub fn new(x: i32, radix: u32) -> Result<Self, &'static str> {
        if radix < 2 || radix > 36 {
            Err("Unsupported radix")
        } else {
            Ok(Self { x, radix })
        }
    }
}

use std::fmt;

impl fmt::Display for Radix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut x = self.x;
        // Good for binary formatting of `u128`s
        let mut result = ['\0'; 128];
        let mut used = 0;
        let negative = x < 0;
        if negative {
            x *= -1;
        }
        let mut x = x as u32;
        loop {
            let m = x % self.radix;
            x /= self.radix;

            result[used] = std::char::from_digit(m, self.radix).unwrap();
            used += 1;

            if x == 0 {
                break;
            }
        }

        if negative {
            write!(f, "-")?;
        }

        for c in result[..used].iter().rev() {
            write!(f, "{}", c)?;
        }

        Ok(())
    }
}
