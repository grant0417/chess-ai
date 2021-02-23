use crate::board::{Board, UNICODE_PIECES};

pub fn print_board(board: &Board) {
    for rank in (0..8).rev() {
        print!(" {} ", rank + 1);
        for file in 0..8 {
            let light_square = (file + rank) % 2 != 0;

            if light_square {
                print!("\x1B[48;2;255;206;158m")
            } else {
                print!("\x1B[48;2;209;139;71m")
            }

            print!("\x1B[38;2;0;0;0m");

            match board.pieces[rank * 8 + file] {
                Some((color, piece)) => {
                    print!(" {} ", UNICODE_PIECES[color as usize][piece as usize])
                }
                None => print!("   "),
            }
        }
        print!("\x1B[0m\n");
    }
    print!("  ");
    for i in 0..8 {
        print!("  {}", (b'a' + i) as char);
    }
    println!("\n");
}

pub const fn algebraic_to_index(s: &[u8]) -> Result<usize, &'static str> {
    if s.len() != 2 {
        return Err("len = 2");
    }

    let file = match s[0] {
        f if f.is_ascii_lowercase() && (f < b'a' || f > b'h') => return Err("Alpha not in range"),
        f if f.is_ascii_uppercase() && (f < b'A' || f > b'H') => return Err("Alpha not in range"),
        f if f.is_ascii_lowercase() => f - b'a',
        f if f.is_ascii_uppercase() => f - b'A',
        _ => return Err(""),
    };

    let rank = match s[1] {
        r if r.is_ascii_digit() && (r > b'8' || r < b'1') => return Err("Numeric not in range"),
        r if r.is_ascii_digit() => r - b'1',
        _ => return Err(""),
    };

    let index = rank as usize * 8 + file as usize;

    if index >= 64 {
        Err("")
    } else {
        Ok(index)
    }
}

pub const fn index_to_algebraic(value: usize) -> [u8; 2] {
    // TODO:
    // if file * 8 + rank >= 64 {
    //      return Err(???)
    // }

    let file = (value as u8 % 8) + b'a';
    let rank = (value as u8 / 8) + b'1';

    [file, rank]
}

#[cfg(test)]
mod test {
    use crate::interface::algebraic_to_index;

    #[test]
    fn test_algebraic_to_index() {
        assert_eq!(algebraic_to_index("a1".as_bytes()).unwrap(), 0);
        assert_eq!(algebraic_to_index("A1".as_bytes()).unwrap(), 0);
        assert_eq!(algebraic_to_index("H8".as_bytes()).unwrap(), 63);
        assert_eq!(algebraic_to_index("h8".as_bytes()).unwrap(), 63);
    }
}
