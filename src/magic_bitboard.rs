// This magic implementation is based on F.M.H. Reul's New architectures in computer chess
// https://pure.uvt.nl/ws/portalfiles/portal/1098572/Proefschrift_Fritz_Reul_170609.pdf#page=126

use crate::util::Rng;

const ROOK_MASK: [u64; 64] = {
    let mut masks = [0; 64];
    let mut i = 0;
    while i < 64 {
        masks[i] = generate_rook_mask(i as i64);
        i += 1;
    }
    masks
};

const BISHOP_MASK: [u64; 64] = {
    let mut masks = [0; 64];
    let mut i = 0;
    while i < 64 {
        masks[i] = generate_bishop_mask(i as i64, 0);
        i += 1;
    }
    masks
};

const ROOK_ATTACKS: [u64; 64] = {
    let mut attacks = [0; 64];
    let mut i = 0;
    while i < 64 {
        attacks[i] = generate_rook_attack(i as i64, 0);
        i += 1;
    }
    attacks
};

const BISHOP_ATTACKS: [u64; 64] = {
    let mut attacks = [0; 64];
    let mut i = 0;
    while i < 64 {
        attacks[i] = generate_bishop_attack(i as i64, 0);
        i += 1;
    }
    attacks
};

const MAGIC_ATTACK_TABLE: [u64; 100000] = {
    let a = [0; 100000];
    a
};

const MAGIC_BISHOP_TABLE: [MagicRecord; 64] = {
    [MagicRecord {
        index: 0,
        mask: 0,
        magic: 0,
        shift: 0,
    }; 64]
};

const MAGIC_ROOK_TABLE: [MagicRecord; 64] = {
    [MagicRecord {
        index: 0,
        mask: 0,
        magic: 0,
        shift: 0,
    }; 64]
};

#[derive(Copy, Clone, Debug)]
struct MagicRecord {
    index: u32,
    mask: u64,
    magic: u64,
    shift: u8,
}

fn bishop_attacks(occupied: u64, square_index: usize) -> u64 {
    let record = MAGIC_BISHOP_TABLE[square_index];
    let mut occ = occupied & record.mask;
    occ *= record.magic;
    occ >>= record.shift;
    occ += record.index as u64;
    MAGIC_ATTACK_TABLE[occ as usize]
}

fn rook_attacks(occupied: u64, square_index: usize) -> u64 {
    let record = MAGIC_ROOK_TABLE[square_index];
    let mut occ = occupied & record.mask;
    occ *= record.magic;
    occ >>= record.shift;
    occ += record.index as u64;
    MAGIC_ATTACK_TABLE[occ as usize]
}

const fn generate_rook_mask(index: i64) -> u64 {
    let mut bitboard = 0i64;
    let file = index % 8;
    let rank = index / 8;

    let mut i = rank + 1;
    while i <= 6 {
        bitboard |= 1 << (file + i * 8);
        i += 1;
    }

    i = rank - 1;
    while i >= 1 {
        bitboard |= 1 << (file + i * 8);
        i -= 1;
    }

    i = file + 1;
    while i <= 6 {
        bitboard |= 1 << (i + rank * 8);
        i += 1;
    }

    i = file - 1;
    while i >= 1 {
        bitboard |= 1 << (i + rank * 8);
        i -= 1;
    }

    bitboard as u64
}

const fn generate_bishop_mask(index: i64, _blockers: u64) -> u64 {
    let mut bitboard = 0i64;
    let file = index % 8;
    let rank = index / 8;

    let mut r = rank + 1;
    let mut f = file + 1;
    while r <= 6 && f <= 6 {
        bitboard |= 1 << (f + r * 8);
        r += 1;
        f += 1;
    }

    r = rank - 1;
    f = file - 1;
    while r >= 1 && f >= 1 {
        bitboard |= 1 << (f + r * 8);
        r -= 1;
        f -= 1;
    }

    r = rank + 1;
    f = file - 1;
    while r <= 6 && f >= 1 {
        bitboard |= 1 << (f + r * 8);
        r += 1;
        f -= 1;
    }

    r = rank - 1;
    f = file + 1;
    while r >= 1 && f <= 6 {
        bitboard |= 1 << (f + r * 8);
        r -= 1;
        f += 1;
    }

    bitboard as u64
}

const fn generate_rook_attack(index: i64, blockers: u64) -> u64 {
    let mut bitboard = 0i64;
    let file = index % 8;
    let rank = index / 8;

    let mut i = rank + 1;
    while i <= 7 {
        bitboard |= 1 << (file + i * 8);
        if blockers & (1 << (file + i * 8)) != 0 {
            break;
        };
        i += 1;
    }

    i = rank - 1;
    while i >= 0 {
        bitboard |= 1 << (file + i * 8);
        if blockers & (1 << (file + i * 8)) != 0 {
            break;
        };
        i -= 1;
    }

    i = file + 1;
    while i <= 7 {
        bitboard |= 1 << (i + rank * 8);
        if blockers & (1 << (file + i * 8)) != 0 {
            break;
        };
        i += 1;
    }

    i = file - 1;
    while i >= 0 {
        bitboard |= 1 << (i + rank * 8);
        if blockers & (1 << (file + i * 8)) != 0 {
            break;
        };
        i -= 1;
    }

    bitboard as u64
}

const fn generate_bishop_attack(index: i64, blockers: u64) -> u64 {
    let mut bitboard = 0i64;
    let file = index % 8;
    let rank = index / 8;

    let mut r = rank + 1;
    let mut f = file + 1;
    while r <= 7 && f <= 7 {
        bitboard |= 1 << (f + r * 8);
        if blockers & (1 << (f + r * 8)) != 0 {
            break;
        };
        r += 1;
        f += 1;
    }

    r = rank - 1;
    f = file - 1;
    while r >= 0 && f >= 0 {
        bitboard |= 1 << (f + r * 8);
        if blockers & (1 << (f + r * 8)) != 0 {
            break;
        };
        r -= 1;
        f -= 1;
    }

    r = rank + 1;
    f = file - 1;
    while r <= 7 && f >= 0 {
        bitboard |= 1 << (f + r * 8);
        if blockers & (1 << (f + r * 8)) != 0 {
            break;
        };
        r += 1;
        f -= 1;
    }

    r = rank - 1;
    f = file + 1;
    while r >= 0 && f <= 7 {
        bitboard |= 1 << (f + r * 8);
        if blockers & (1 << (f + r * 8)) != 0 {
            break;
        };
        r -= 1;
        f += 1;
    }

    bitboard as u64
}

const fn magic_function(isolated_bit: u64, magic: u64, size: usize) -> usize {
    ((isolated_bit * magic) >> (64 - size)) as usize
}

const fn find_magic_multiplier(index: usize, _one_bits: usize) -> u64 {
    let blockers = [0; 4096];
    let mut solution = [0; 4096];

    let mask = generate_rook_mask(index as i64);
    let bits = mask.count_ones() as usize;

    let mut i = 0;
    while i < (1 << bits) {
        //blockers[i] = generate_rook_attack(i as i64, blockers[i]);
        solution[i] = generate_rook_attack(i as i64, blockers[i]);
        i += 1;
    }
    loop {
        let _used = [0; 4096];
        let magic = 0;
        let failed = false;

        let mut i = 0;
        while i < (1 << bits) {
            let _index = magic_function(blockers[i], magic, bits);
            i += 1;
        }
        if failed == false {
            return magic;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::bitboard::print_bitboard;
    use crate::magic_bitboard::generate_bishop_mask;

    #[test]
    fn test_bishop() {}
}
