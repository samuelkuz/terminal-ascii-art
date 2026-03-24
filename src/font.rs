pub type Glyph = [&'static str; 5];

pub trait Font {
    fn name(&self) -> &'static str;
    fn height(&self) -> usize;
    fn glyph(&self, ch: char) -> &'static Glyph;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StandardFont;

impl Font for StandardFont {
    fn name(&self) -> &'static str {
        "standard"
    }

    fn height(&self) -> usize {
        5
    }

    fn glyph(&self, ch: char) -> &'static Glyph {
        match ch.to_ascii_uppercase() {
            'A' => &A,
            'B' => &B,
            'C' => &C,
            'D' => &D,
            'E' => &E,
            'F' => &F,
            'G' => &G,
            'H' => &H,
            'I' => &I,
            'J' => &J,
            'K' => &K,
            'L' => &L,
            'M' => &M,
            'N' => &N,
            'O' => &O,
            'P' => &P,
            'Q' => &Q,
            'R' => &R,
            'S' => &S,
            'T' => &T,
            'U' => &U,
            'V' => &V,
            'W' => &W,
            'X' => &X,
            'Y' => &Y,
            'Z' => &Z,
            '0' => &ZERO,
            '1' => &ONE,
            '2' => &TWO,
            '3' => &THREE,
            '4' => &FOUR,
            '5' => &FIVE,
            '6' => &SIX,
            '7' => &SEVEN,
            '8' => &EIGHT,
            '9' => &NINE,
            ' ' => &SPACE,
            '!' => &EXCLAMATION,
            '?' => &QUESTION,
            '.' => &PERIOD,
            ',' => &COMMA,
            ':' => &COLON,
            ';' => &SEMICOLON,
            '-' => &HYPHEN,
            '_' => &UNDERSCORE,
            '/' => &SLASH,
            '\\' => &BACKSLASH,
            '\'' => &APOSTROPHE,
            '"' => &QUOTE,
            _ => &QUESTION,
        }
    }
}

const A: Glyph = [" ### ", "#   #", "#####", "#   #", "#   #"];
const B: Glyph = ["#### ", "#   #", "#### ", "#   #", "#### "];
const C: Glyph = [" ####", "#    ", "#    ", "#    ", " ####"];
const D: Glyph = ["#### ", "#   #", "#   #", "#   #", "#### "];
const E: Glyph = ["#####", "#    ", "#### ", "#    ", "#####"];
const F: Glyph = ["#####", "#    ", "#### ", "#    ", "#    "];
const G: Glyph = [" ####", "#    ", "#  ##", "#   #", " ####"];
const H: Glyph = ["#   #", "#   #", "#####", "#   #", "#   #"];
const I: Glyph = ["#####", "  #  ", "  #  ", "  #  ", "#####"];
const J: Glyph = ["#####", "   # ", "   # ", "#  # ", " ##  "];
const K: Glyph = ["#   #", "#  # ", "###  ", "#  # ", "#   #"];
const L: Glyph = ["#    ", "#    ", "#    ", "#    ", "#####"];
const M: Glyph = ["#   #", "## ##", "# # #", "#   #", "#   #"];
const N: Glyph = ["#   #", "##  #", "# # #", "#  ##", "#   #"];
const O: Glyph = [" ### ", "#   #", "#   #", "#   #", " ### "];
const P: Glyph = ["#### ", "#   #", "#### ", "#    ", "#    "];
const Q: Glyph = [" ### ", "#   #", "#   #", "#  ##", " ####"];
const R: Glyph = ["#### ", "#   #", "#### ", "#  # ", "#   #"];
const S: Glyph = [" ####", "#    ", " ### ", "    #", "#### "];
const T: Glyph = ["#####", "  #  ", "  #  ", "  #  ", "  #  "];
const U: Glyph = ["#   #", "#   #", "#   #", "#   #", " ### "];
const V: Glyph = ["#   #", "#   #", "#   #", " # # ", "  #  "];
const W: Glyph = ["#   #", "#   #", "# # #", "## ##", "#   #"];
const X: Glyph = ["#   #", " # # ", "  #  ", " # # ", "#   #"];
const Y: Glyph = ["#   #", " # # ", "  #  ", "  #  ", "  #  "];
const Z: Glyph = ["#####", "   # ", "  #  ", " #   ", "#####"];
const ZERO: Glyph = [" ### ", "#  ##", "# # #", "##  #", " ### "];
const ONE: Glyph = ["  #  ", " ##  ", "  #  ", "  #  ", " ### "];
const TWO: Glyph = [" ### ", "#   #", "   # ", "  #  ", "#####"];
const THREE: Glyph = [" ### ", "    #", " ### ", "    #", " ### "];
const FOUR: Glyph = ["#   #", "#   #", "#####", "    #", "    #"];
const FIVE: Glyph = ["#####", "#    ", "#### ", "    #", "#### "];
const SIX: Glyph = [" ####", "#    ", "#### ", "#   #", " ### "];
const SEVEN: Glyph = ["#####", "    #", "   # ", "  #  ", " #   "];
const EIGHT: Glyph = [" ### ", "#   #", " ### ", "#   #", " ### "];
const NINE: Glyph = [" ### ", "#   #", " ####", "    #", " ### "];
const SPACE: Glyph = ["     ", "     ", "     ", "     ", "     "];
const EXCLAMATION: Glyph = ["  #  ", "  #  ", "  #  ", "     ", "  #  "];
const QUESTION: Glyph = [" ### ", "#   #", "  ## ", "     ", "  #  "];
const PERIOD: Glyph = ["     ", "     ", "     ", "     ", "  #  "];
const COMMA: Glyph = ["     ", "     ", "     ", "  #  ", " #   "];
const COLON: Glyph = ["     ", "  #  ", "     ", "  #  ", "     "];
const SEMICOLON: Glyph = ["     ", "  #  ", "     ", "  #  ", " #   "];
const HYPHEN: Glyph = ["     ", "     ", " ### ", "     ", "     "];
const UNDERSCORE: Glyph = ["     ", "     ", "     ", "     ", "#####"];
const SLASH: Glyph = ["    #", "   # ", "  #  ", " #   ", "#    "];
const BACKSLASH: Glyph = ["#    ", " #   ", "  #  ", "   # ", "    #"];
const APOSTROPHE: Glyph = ["  #  ", "  #  ", " #   ", "     ", "     "];
const QUOTE: Glyph = [" # # ", " # # ", "     ", "     ", "     "];

#[cfg(test)]
mod tests {
    use super::{Font, StandardFont};

    #[test]
    fn unknown_character_uses_fallback_glyph() {
        let font = StandardFont;

        assert_eq!(font.glyph('ß'), font.glyph('?'));
    }

    #[test]
    fn lowercase_maps_to_uppercase_glyph() {
        let font = StandardFont;

        assert_eq!(font.glyph('a'), font.glyph('A'));
    }
}
