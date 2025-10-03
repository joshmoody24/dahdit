// Morse code pattern lookup table - O(1) character-to-pattern mapping
// 0 = dot, 1 = dash, pattern ends at first None
use crate::types::MorseElementType;

pub type MorsePattern = &'static [MorseElementType];

const DOT: MorseElementType = MorseElementType::Dot;
const DASH: MorseElementType = MorseElementType::Dash;

// Letter patterns
const PATTERN_A: MorsePattern = &[DOT, DASH]; // .-
const PATTERN_B: MorsePattern = &[DASH, DOT, DOT, DOT]; // -...
const PATTERN_C: MorsePattern = &[DASH, DOT, DASH, DOT]; // -.-.
const PATTERN_D: MorsePattern = &[DASH, DOT, DOT]; // -..
const PATTERN_E: MorsePattern = &[DOT]; // .
const PATTERN_F: MorsePattern = &[DOT, DOT, DASH, DOT]; // ..-.
const PATTERN_G: MorsePattern = &[DASH, DASH, DOT]; // --.
const PATTERN_H: MorsePattern = &[DOT, DOT, DOT, DOT]; // ....
const PATTERN_I: MorsePattern = &[DOT, DOT]; // ..
const PATTERN_J: MorsePattern = &[DOT, DASH, DASH, DASH]; // .---
const PATTERN_K: MorsePattern = &[DASH, DOT, DASH]; // -.-
const PATTERN_L: MorsePattern = &[DOT, DASH, DOT, DOT]; // .-..
const PATTERN_M: MorsePattern = &[DASH, DASH]; // --
const PATTERN_N: MorsePattern = &[DASH, DOT]; // -.
const PATTERN_O: MorsePattern = &[DASH, DASH, DASH]; // ---
const PATTERN_P: MorsePattern = &[DOT, DASH, DASH, DOT]; // .--.
const PATTERN_Q: MorsePattern = &[DASH, DASH, DOT, DASH]; // --.-
const PATTERN_R: MorsePattern = &[DOT, DASH, DOT]; // .-.
const PATTERN_S: MorsePattern = &[DOT, DOT, DOT]; // ...
const PATTERN_T: MorsePattern = &[DASH]; // -
const PATTERN_U: MorsePattern = &[DOT, DOT, DASH]; // ..-
const PATTERN_V: MorsePattern = &[DOT, DOT, DOT, DASH]; // ...-
const PATTERN_W: MorsePattern = &[DOT, DASH, DASH]; // .--
const PATTERN_X: MorsePattern = &[DASH, DOT, DOT, DASH]; // -..-
const PATTERN_Y: MorsePattern = &[DASH, DOT, DASH, DASH]; // -.--
const PATTERN_Z: MorsePattern = &[DASH, DASH, DOT, DOT]; // --..

// Number patterns
const PATTERN_0: MorsePattern = &[DASH, DASH, DASH, DASH, DASH]; // -----
const PATTERN_1: MorsePattern = &[DOT, DASH, DASH, DASH, DASH]; // .----
const PATTERN_2: MorsePattern = &[DOT, DOT, DASH, DASH, DASH]; // ..---
const PATTERN_3: MorsePattern = &[DOT, DOT, DOT, DASH, DASH]; // ...--
const PATTERN_4: MorsePattern = &[DOT, DOT, DOT, DOT, DASH]; // ....-
const PATTERN_5: MorsePattern = &[DOT, DOT, DOT, DOT, DOT]; // .....
const PATTERN_6: MorsePattern = &[DASH, DOT, DOT, DOT, DOT]; // -....
const PATTERN_7: MorsePattern = &[DASH, DASH, DOT, DOT, DOT]; // --...
const PATTERN_8: MorsePattern = &[DASH, DASH, DASH, DOT, DOT]; // ---..
const PATTERN_9: MorsePattern = &[DASH, DASH, DASH, DASH, DOT]; // ----.

// Punctuation patterns
const PATTERN_PERIOD: MorsePattern = &[DOT, DASH, DOT, DASH, DOT, DASH]; // .-.-.-
const PATTERN_COMMA: MorsePattern = &[DASH, DASH, DOT, DOT, DASH, DASH]; // --..--
const PATTERN_QUESTION: MorsePattern = &[DOT, DOT, DASH, DASH, DOT, DOT]; // ..--..
const PATTERN_QUOTE: MorsePattern = &[DOT, DASH, DASH, DASH, DASH, DOT]; // .----.
const PATTERN_EXCLAIM: MorsePattern = &[DASH, DOT, DASH, DOT, DASH, DASH]; // -.-.--
const PATTERN_SLASH: MorsePattern = &[DASH, DOT, DOT, DASH, DOT]; // -..-.
const PATTERN_LPAREN: MorsePattern = &[DASH, DOT, DASH, DASH, DOT]; // -.--.
const PATTERN_RPAREN: MorsePattern = &[DASH, DOT, DASH, DASH, DOT, DASH]; // -.--.-
const PATTERN_AMPERSAND: MorsePattern = &[DOT, DASH, DOT, DOT, DOT]; // .-...
const PATTERN_COLON: MorsePattern = &[DASH, DASH, DASH, DOT, DOT, DOT]; // ---...
const PATTERN_SEMICOLON: MorsePattern = &[DASH, DOT, DASH, DOT, DASH, DOT]; // -.-.-.
const PATTERN_EQUALS: MorsePattern = &[DASH, DOT, DOT, DOT, DASH]; // -...-
const PATTERN_PLUS: MorsePattern = &[DOT, DASH, DOT, DASH, DOT]; // .-.-.
const PATTERN_HYPHEN: MorsePattern = &[DASH, DOT, DOT, DOT, DOT, DASH]; // -....-
const PATTERN_UNDERSCORE: MorsePattern = &[DOT, DOT, DASH, DASH, DOT, DASH]; // ..--.-
const PATTERN_DQUOTE: MorsePattern = &[DOT, DASH, DOT, DOT, DASH, DOT]; // .-..-.
const PATTERN_DOLLAR: MorsePattern = &[DOT, DOT, DOT, DASH, DOT, DOT, DASH]; // ...-..-
const PATTERN_AT: MorsePattern = &[DOT, DASH, DASH, DOT, DASH, DOT]; // .--.-.

// Direct lookup table for O(1) access - 256 entries for all possible bytes
static MORSE_PATTERNS: [Option<MorsePattern>; 256] = {
    let mut patterns = [None; 256];

    // Uppercase letters
    patterns[b'A' as usize] = Some(PATTERN_A);
    patterns[b'B' as usize] = Some(PATTERN_B);
    patterns[b'C' as usize] = Some(PATTERN_C);
    patterns[b'D' as usize] = Some(PATTERN_D);
    patterns[b'E' as usize] = Some(PATTERN_E);
    patterns[b'F' as usize] = Some(PATTERN_F);
    patterns[b'G' as usize] = Some(PATTERN_G);
    patterns[b'H' as usize] = Some(PATTERN_H);
    patterns[b'I' as usize] = Some(PATTERN_I);
    patterns[b'J' as usize] = Some(PATTERN_J);
    patterns[b'K' as usize] = Some(PATTERN_K);
    patterns[b'L' as usize] = Some(PATTERN_L);
    patterns[b'M' as usize] = Some(PATTERN_M);
    patterns[b'N' as usize] = Some(PATTERN_N);
    patterns[b'O' as usize] = Some(PATTERN_O);
    patterns[b'P' as usize] = Some(PATTERN_P);
    patterns[b'Q' as usize] = Some(PATTERN_Q);
    patterns[b'R' as usize] = Some(PATTERN_R);
    patterns[b'S' as usize] = Some(PATTERN_S);
    patterns[b'T' as usize] = Some(PATTERN_T);
    patterns[b'U' as usize] = Some(PATTERN_U);
    patterns[b'V' as usize] = Some(PATTERN_V);
    patterns[b'W' as usize] = Some(PATTERN_W);
    patterns[b'X' as usize] = Some(PATTERN_X);
    patterns[b'Y' as usize] = Some(PATTERN_Y);
    patterns[b'Z' as usize] = Some(PATTERN_Z);

    // Lowercase letters (same patterns)
    patterns[b'a' as usize] = Some(PATTERN_A);
    patterns[b'b' as usize] = Some(PATTERN_B);
    patterns[b'c' as usize] = Some(PATTERN_C);
    patterns[b'd' as usize] = Some(PATTERN_D);
    patterns[b'e' as usize] = Some(PATTERN_E);
    patterns[b'f' as usize] = Some(PATTERN_F);
    patterns[b'g' as usize] = Some(PATTERN_G);
    patterns[b'h' as usize] = Some(PATTERN_H);
    patterns[b'i' as usize] = Some(PATTERN_I);
    patterns[b'j' as usize] = Some(PATTERN_J);
    patterns[b'k' as usize] = Some(PATTERN_K);
    patterns[b'l' as usize] = Some(PATTERN_L);
    patterns[b'm' as usize] = Some(PATTERN_M);
    patterns[b'n' as usize] = Some(PATTERN_N);
    patterns[b'o' as usize] = Some(PATTERN_O);
    patterns[b'p' as usize] = Some(PATTERN_P);
    patterns[b'q' as usize] = Some(PATTERN_Q);
    patterns[b'r' as usize] = Some(PATTERN_R);
    patterns[b's' as usize] = Some(PATTERN_S);
    patterns[b't' as usize] = Some(PATTERN_T);
    patterns[b'u' as usize] = Some(PATTERN_U);
    patterns[b'v' as usize] = Some(PATTERN_V);
    patterns[b'w' as usize] = Some(PATTERN_W);
    patterns[b'x' as usize] = Some(PATTERN_X);
    patterns[b'y' as usize] = Some(PATTERN_Y);
    patterns[b'z' as usize] = Some(PATTERN_Z);

    // Numbers
    patterns[b'0' as usize] = Some(PATTERN_0);
    patterns[b'1' as usize] = Some(PATTERN_1);
    patterns[b'2' as usize] = Some(PATTERN_2);
    patterns[b'3' as usize] = Some(PATTERN_3);
    patterns[b'4' as usize] = Some(PATTERN_4);
    patterns[b'5' as usize] = Some(PATTERN_5);
    patterns[b'6' as usize] = Some(PATTERN_6);
    patterns[b'7' as usize] = Some(PATTERN_7);
    patterns[b'8' as usize] = Some(PATTERN_8);
    patterns[b'9' as usize] = Some(PATTERN_9);

    // Punctuation
    patterns[b'.' as usize] = Some(PATTERN_PERIOD);
    patterns[b',' as usize] = Some(PATTERN_COMMA);
    patterns[b'?' as usize] = Some(PATTERN_QUESTION);
    patterns[b'\'' as usize] = Some(PATTERN_QUOTE);
    patterns[b'!' as usize] = Some(PATTERN_EXCLAIM);
    patterns[b'/' as usize] = Some(PATTERN_SLASH);
    patterns[b'(' as usize] = Some(PATTERN_LPAREN);
    patterns[b')' as usize] = Some(PATTERN_RPAREN);
    patterns[b'&' as usize] = Some(PATTERN_AMPERSAND);
    patterns[b':' as usize] = Some(PATTERN_COLON);
    patterns[b';' as usize] = Some(PATTERN_SEMICOLON);
    patterns[b'=' as usize] = Some(PATTERN_EQUALS);
    patterns[b'+' as usize] = Some(PATTERN_PLUS);
    patterns[b'-' as usize] = Some(PATTERN_HYPHEN);
    patterns[b'_' as usize] = Some(PATTERN_UNDERSCORE);
    patterns[b'"' as usize] = Some(PATTERN_DQUOTE);
    patterns[b'$' as usize] = Some(PATTERN_DOLLAR);
    patterns[b'@' as usize] = Some(PATTERN_AT);

    patterns
};

/// Get morse pattern for a character - O(1) lookup
pub fn get_morse_pattern(ch: u8) -> Option<MorsePattern> {
    MORSE_PATTERNS[ch as usize]
}
