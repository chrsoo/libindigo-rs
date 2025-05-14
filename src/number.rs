
//! From the INDI INDI Protocol [reference documentation](https://docs.indilib.org/protocol/INDI.pdf):
//!
//! > The format of a numberValue shall be any one of integer, real or sexagesimal;
//! > each sexagesimal separator shall be any one of space( ), colon (:) or
//! > semicolon (;); each sexagesimal component specified shall be integer or real;
//! > unspecified components shall default to 0; negative values shall be indicated
//! > with a leading hyphen (-). For example, the following are all the same
//! > numeric value: "-10:30:18", "-10 30.3" and "-10.505".

use core::{slice, str};
use bitfield::bitfield;
use funty::Numeric;
use log::warn;
use std::{ffi::c_char, fmt::Display, num::{ParseFloatError, ParseIntError}, str::{FromStr, Utf8Error}};

const SEXAGESIMAL_FRACTIONS: &[usize] = &[3, 5, 6, 8, 9];

const FMT_ERR_TRAILING_CHARACTERS_AFTER_FORMAT: &str = "trailing character(s) after format";
const FMT_ERR_FIRST_CHARACTER_MUST_BE_A_PERCENT_SIGN: &str = "first character must be a percent sign";
const FMT_ERR_FORMAT_MUST_BE_ZERO_TERMINATED: &str = "format must be \0 terminated";
const FMT_ERR_FORMAT_MUST_ONLY_CONTAIN_ONE_PERIOD: &str = "format must only contain one period";
const FMT_ERR_LENGTH: &str = "length error";
const FMT_ERR_NO_FORMAT_SPECIFIER: &str = "no format specifier";
const FMT_ERR_PREMATURE_END_OF_FORMAT_STRING: &str = "premature end of format string";
const FMT_ERR_ZERO_LENGTH_BUFFER: &str = "zero length buffer";
const FMT_ERR_SEXAGESIMAL_FRACTION_MUST_BE_3_5_6_8_OR_9: &str = "sexagesimal fraction must be 3, 5, 6, 8 or 9";
const FMT_ERR_SEXAGESIMAL_FORMAT_MUST_SPECIFY_WIDTH: &str = "sexagesimal format must spefify width";
const FMT_ERR_SEXAGESIMAL_FORMAT_MUST_SPECIFY_PRECISION: &str = "sexagesimal format must spefify precision";
const FMT_ERR_SEXAGESIMAL_WIDTH_MUST_EQUAL_OR_EXCEED_PRECISION: &str =
    "sexagesimal width must equal or exceed precision";
const FMT_ERR_UNKNOWN_FORMAT_SPECIFIER: &str = "unknown format specifier";
const FMT_ERR_UNSUPORTED_PRINTF_FORMAT: &str = "unsuported printf format";

pub fn format_sexagesimal(fmt: &NumberFormat, nbr: f64) -> String {
    match fmt.p {
        Some(3) => format!("{}:{:02.0}", nbr.trunc(), (nbr.fract() * 60f64)),
        Some(5) => format!("{}:{:04.01}", nbr.trunc(), (nbr.fract() * 60f64)),
        Some(6) => format!("{}:{:02}:{:02.0}", nbr.trunc(), (nbr.fract() * 60f64).trunc().abs(), ((nbr.fract() * 60f64).fract() * 60f64).abs()),
        Some(8) => format!("{}:{:02}:{:04.01}", nbr.trunc(), (nbr.fract() * 60f64).trunc().abs(), ((nbr.fract() * 60f64).fract() * 60f64).abs()),
        Some(9) => format!("{}:{:02}:{:05.02}", nbr.trunc(), (nbr.fract() * 60f64).trunc().abs(), ((nbr.fract() * 60f64).fract() * 60f64).abs()),
        p       => unreachable!("expected sexagesimal precison of Some(3|5|6|8|9) but was {p:?}", ),
    }
}

pub fn parse_sexagesimal(s: &str) -> Result<f64,ParseError> {
    let bytes = s.as_bytes();
    let mut j = 0;
    while j < bytes.len() {
        match bytes[j] as char {
            '0'..='9'|'.'|'-'|'+'   => j += 1,
            ' '|':'|';'             => break,
            _                       => return Err(ParseError::new("illegal sexagesimal character", j)),
        }
    }
    let nbr = parse_float(bytes, 0, j)?;
    let sign = if bytes[0] as char == '-' { -1f64 } else { 1f64 };

    match bytes.len() - j {
        0   => Ok(nbr),
        3|5 => Ok(nbr + sign * parse_float(bytes, j+1, bytes.len())? / 60f64),
        6   => if bytes[j+3] as char == '.' {
            Ok(nbr + sign * parse_float(bytes, j+1, bytes.len())? / 60f64)
        } else {
            Ok(nbr + sign * parse_float(bytes, j+1, j+3)? / 60f64 + sign * parse_float(bytes, j+4, bytes.len())? / 3600f64)
        },
        8|9 => Ok(nbr + sign * parse_float(bytes, j+1, j+3)? / 60f64 + sign * parse_float(bytes, j+4, bytes.len())? / 3600f64),
        _   => Err(ParseError::new(FMT_ERR_SEXAGESIMAL_FRACTION_MUST_BE_3_5_6_8_OR_9, j))
    }
}

fn parse_float(bytes: &[u8], i: usize, j: usize) -> Result<f64,ParseError<'static>> {
    if i == j { return Err(ParseError::new("begin and start indeces must not be equal", i)) }
    if i > j { return Err(ParseError::new("begin index must smaller than end index", i)) }
    let utf_str = str::from_utf8(&bytes[i..j])?;
    Ok(utf_str.parse()?)
}

/// A `printf` C-style format specifier with support for the INDI sexagesimal extension.
/// NumberFormat is based on the following template:
/// > ```text
/// > %[flags][width][.precision][length]specifier
/// > ```
/// Unsupported  features of [printf](https://cplusplus.com/reference/cstdio/printf/):
/// * `.*` (unspecified precision)
/// * `cspn%` (format specifiers)
///
/// Note: the `NumberFormat` string is expected to be stand-alone, i.e. it must not be embedded
/// in a longer string. The following formats are valid:
/// ```
/// use libindigo::NumberFormat;
/// "&5f".parse::<NumberFormat>();
/// "&05.1f".parse::<NumberFormat>();
/// "&12.6m".parse::<NumberFormat>();
/// ```
/// Sexagesimals use the `m` specifier and are defined as follows, from the INDI Protocol
/// [reference documentation](https://docs.indilib.org/protocol/INDI.pdf):
/// > A numberFormat shall be any string that includes exactly one printf-style
/// > format specification appropriate for C-type double or one INDI style "m" to
/// > specify sexagesimal in the form `%<w>.<f>m` where
/// > ```text
/// >    <w> is the total field width
/// >    <f> is the width of the fraction. valid values are:
/// >        9  ->  :mm:ss.ss
/// >        8  ->  :mm:ss.s
/// >        6  ->  :mm:ss
/// >        5  ->  :mm.m
/// >        3  ->  :mm
/// > ```
/// > For example:
/// > ```text
/// >    to produce...   use...
/// >        "-123:45"   %7.3m
/// >        "  0:01:02" %9.6m
/// > ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NumberFormat {
    f: Option<FormatFlags>,
    w: Option<usize>,
    p: Option<usize>,
    l: Option<FormatLength>,
    s: char,
}

impl NumberFormat {
    /// Create a new `NumberFormat` from a `&str`, panicing if the format is not valid.
    pub fn new(fmt: &str) -> Self {
        fmt.parse().unwrap()
    }

    pub fn format(&self, n: impl Numeric) -> String {
        match self.s {
            'm' => format_sexagesimal(self, n.as_f64()),
            _   => self.format_number(n),
        }
    }

    pub fn format_number(&self, n: impl Numeric) -> String {
        match self {
            NumberFormat { f: Some(FormatFlags(FormatFlags::ZERO)), w: Some(w), p: None, l: None, s: _ } => format!("{n:0w$}"),
            NumberFormat { f: Some(FormatFlags(FormatFlags::ZERO)), w: Some(w), p: Some(p), l: None, s: _ } => format!("{n:0w$.p$}"),
            NumberFormat { f: Some(FormatFlags(FormatFlags::PLUS)), w: None, p: None, l: None, s: _ } => format!("{n:+}"),
            NumberFormat { f: Some(FormatFlags(FormatFlags::PLUS)), w: Some(w), p: None, l: None, s: _ } => format!("{n:+w$}"),
            NumberFormat { f: Some(FormatFlags(FormatFlags::PLUS)), w: Some(w), p: Some(p), l: None, s: _ } => format!("{n:+w$.p$}"),
            NumberFormat { f: _, w: Some(w), p: None, l: None, s: _ } => format!("{n:w$}"),
            NumberFormat { f: _, w: Some(w), p: Some(p), l: None, s: _ } => format!("{n:w$.p$}"),
            NumberFormat { f: _, w: None, p: Some(p), l: None, s: _ } => format!("{n:.p$}"),
            NumberFormat { f: _, w: None, p: None, l: None, s: _ } => format!("{n}"),
            _ => format!("{n}"),
        }
    }

    fn parse_width(value: &[u8], flags: Option<FormatFlags>, start: usize, index: usize) -> Result<Option<usize>,ParseError<'static>> {
        if index == start {
            if let Some(f) = flags {
                if !f.zero_flag() {
                    return Err(ParseError::new(FMT_ERR_SEXAGESIMAL_FORMAT_MUST_SPECIFY_WIDTH, index))
                }
            } else {
                return Err(ParseError::new(FMT_ERR_SEXAGESIMAL_FORMAT_MUST_SPECIFY_WIDTH, index))
            }
            return Ok(None)
        }
        Ok(Some(parse_format_number(value, start, index)?))
    }
}

impl FromStr for NumberFormat {
    type Err = ParseError<'static>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NumberFormat::try_from(s)
    }
}

impl Display for NumberFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(flags) = self.f {
            write!(f, "{flags}")?;
        }
        if let Some(width) = self.w {
            write!(f, "{width}")?;
        }
        if let Some(precision) = self.p {
            write!(f, ".{precision}")?;
        }
        if let Some(length) = self.l {
            write!(f, "{length}")?;
        }
        write!(f, "{}", self.s)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError<'a> {
    msg: &'a str,
    index: usize,
}

impl<'a> ParseError<'a> {
    fn new(msg: &'a str, index: usize) -> ParseError<'a>{
        ParseError { msg, index }
    }
}

bitfield!{
  #[derive(PartialEq, Eq, Clone, Copy)]
  pub struct FormatFlags(u8);
  impl Debug;
  u8;
  /// Left justify the number within the width.
  mask MINUS(u8), minus_flag, set_minus_flag: 0;
  /// Prefix positive numbers with a plus sign.
  mask PLUS(u8), plus_flag, set_plus_flag: 1;
  /// Prefix with space if no sign is printed.
  mask SPACE(u8), space_flag, set_space_flag: 2;
  /// Prefix octal and hex or always use a decimal point for floats.
  mask HASH(u8), hash_flag, set_hash_flag : 3;
  /// Zero left pad number up until width.
  mask ZERO(u8), zero_flag, set_zero_flag: 4;
}

const UNSET_FORMAT_FLAGS: FormatFlags = FormatFlags(0);

impl Display for FormatFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.minus_flag() {
            write!(f, "-")?;
        }
        if self.plus_flag() {
            write!(f, "+")?;
        }
        if self.space_flag() {
            write!(f, " ")?;
        }
        if self.hash_flag() {
            write!(f, "#")?;
        }
        if self.zero_flag() {
            write!(f, "0")?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[derive(strum_macros::Display)]
#[allow(non_camel_case_types)]
enum FormatLength {
    hh,
    h,
    l,
    ll,
    j,
    z,
    t,
    L,
}

impl<'a> From<Utf8Error> for ParseError<'a> {
    fn from(value: Utf8Error) -> Self {
        ParseError {
            msg: "could not parse UTF-8 string",
            index: value.valid_up_to()
        }
    }
}

impl<'a> From<ParseIntError> for ParseError<'a> {
    fn from(value: ParseIntError) -> Self {
        warn!("{}", value);
        ParseError {
            msg: "could not parse int",
            index: 0,
        }
    }
}
impl<'a> From<ParseFloatError> for ParseError<'a> {
    fn from(value: ParseFloatError) -> Self {
        warn!("{}", value);
        ParseError {
            msg: "could not parse float",
            index: 0,
        }
    }
}

impl<const N: usize> TryFrom<&[c_char; N]> for NumberFormat {
    type Error = ParseError<'static>;

    fn try_from(value: &[c_char; N]) -> Result<Self, Self::Error> {
        let bytes = unsafe{ slice::from_raw_parts(value.as_ptr() as *const u8, N) };
        let utf_str = str::from_utf8(&bytes)?;
        NumberFormat::try_from(utf_str.as_bytes())
    }
}

impl TryFrom<&str> for NumberFormat {
    type Error = ParseError<'static>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes = value.as_bytes();
        NumberFormat::try_from(bytes)
    }

}

impl TryFrom<&[u8]> for NumberFormat {
    type Error = ParseError<'static>;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() == 0 {
            return Err(ParseError::new(FMT_ERR_ZERO_LENGTH_BUFFER, 1))
        }
        if  value[0] != '%' as u8 {
            return Err(ParseError::new(FMT_ERR_FIRST_CHARACTER_MUST_BE_A_PERCENT_SIGN, 0))
        }

        let mut index = 1;

        // parse flags
        let mut flags = FormatFlags(0);
        while index < value.len()  {
            match value[index] as u8 as char {
                // flags
                '-'         => flags.set_minus_flag(true),
                '+'         => flags.set_plus_flag(true),
                ' '         => flags.set_space_flag(true),
                '#'         => flags.set_hash_flag(true),
                '0'         => flags.set_zero_flag(true),
                '\0'        => return Err(ParseError::new(FMT_ERR_PREMATURE_END_OF_FORMAT_STRING, index)),
                _           => break
            }
            index += 1;
        }
        let flags = if flags == UNSET_FORMAT_FLAGS {None} else {Some(flags)} ;

        let mut width = None;
        let mut precision = None;

        // parse width
        let mut start = index;
        let mut dot = false;
        while index < value.len()  {
            match value[index] as char {
                '0'..='9'   => index += 1, // 0-9, continue the loop
                '.'         => {
                    if dot {
                        return Err(ParseError::new(FMT_ERR_FORMAT_MUST_ONLY_CONTAIN_ONE_PERIOD, index))
                    }
                    width = NumberFormat::parse_width(value, flags, start, index)?;
                    dot = true;
                    index += 1;
                    start = index;
                },
                '\0'        => return Err(ParseError::new(FMT_ERR_PREMATURE_END_OF_FORMAT_STRING, index)),
                _           => break
            }
        }
        if index > start {
            if width.is_none() & !dot {
                width = Some(parse_format_number(value, start, index)?);
            } else {
                precision = Some(parse_format_number(value, start, index)?);
            }
        }

        // parse length
        let mut length = None;
        while index < value.len() {
            match value[index] as u8 as char {
                'h' => match length {
                    None                    => length = Some(FormatLength::h),
                    Some(FormatLength::l)   => length = Some(FormatLength::hh),
                    Some(_)                 => return Err(ParseError::new(FMT_ERR_LENGTH, index)),
                },
                'l' => match length {
                    None                    => length = Some(FormatLength::l),
                    Some(FormatLength::l)   => length = Some(FormatLength::ll),
                    Some(_)                 => return Err(ParseError::new(FMT_ERR_LENGTH, index)),
                },
                'j' => match length {
                    None                    => length = Some(FormatLength::j),
                    Some(_)                 => return Err(ParseError::new(FMT_ERR_LENGTH, index)),
                },
                'z' => match length {
                    None                    => length = Some(FormatLength::z),
                    Some(_)                 => return Err(ParseError::new(FMT_ERR_LENGTH, index)),
                },
                't' => match length {
                    None                    => length = Some(FormatLength::t),
                    Some(_)                 => return Err(ParseError::new(FMT_ERR_LENGTH, index)),
                },
                'L' => match length {
                    None                    => length = Some(FormatLength::L),
                    Some(_)                 => return Err(ParseError::new(FMT_ERR_LENGTH, index)),
                },
                '\0'=> return Err(ParseError::new(FMT_ERR_PREMATURE_END_OF_FORMAT_STRING, index)),
                _   => break
            }
            index += 1;
        }

        if index == value.len() {
            return Err(ParseError::new(FMT_ERR_NO_FORMAT_SPECIFIER, index))
        }
        let specifier = value[index] as u8 as char;
        match specifier {
            'd'|'i'                 => (), // decimal, integer
            'u'                     => (), // unsigned integer
            'o'|'x'|'X'             => (), // octal, lower hex, upper hex
            'f'|'F'|'e'|'E'|'g'|'G' => (), // decimal floats
            'a'|'A'                 => (), // hexadeciaml floats
            'm'                     => validate_sexagesimal(index, width, precision)?,
            'c'|'s'|'p'|'n'|'%'     => return Err(ParseError::new(FMT_ERR_UNSUPORTED_PRINTF_FORMAT, index)),
            '\0'                    => return Err(ParseError::new(FMT_ERR_PREMATURE_END_OF_FORMAT_STRING, index)),
            _                       => return Err(ParseError::new(FMT_ERR_UNKNOWN_FORMAT_SPECIFIER, index)),
        }
        index += 1;

        if index != value.len() {
            if value[index] as u8 as char != '\0' {
                return Err(ParseError::new(FMT_ERR_TRAILING_CHARACTERS_AFTER_FORMAT, index))
            }
        }

        Ok(NumberFormat {
            f: flags, w: width, p: precision, l: length, s: specifier
        })
    }
}

fn validate_sexagesimal(i: usize, w: Option<usize>, p: Option<usize>) -> Result<(),ParseError<'static>> {
    if let Some(p) = p {
        if !SEXAGESIMAL_FRACTIONS.contains(&p) {
            return Err(ParseError::new(FMT_ERR_SEXAGESIMAL_FRACTION_MUST_BE_3_5_6_8_OR_9, i))
        }
    } else {
        return Err(ParseError::new(FMT_ERR_SEXAGESIMAL_FORMAT_MUST_SPECIFY_PRECISION, i))
    }

    if let Some(w) = w {
        if w < p.unwrap() {
            return Err(ParseError::new(FMT_ERR_SEXAGESIMAL_WIDTH_MUST_EQUAL_OR_EXCEED_PRECISION, i))
        }
    } else {
        return Err(ParseError::new(FMT_ERR_SEXAGESIMAL_FORMAT_MUST_SPECIFY_WIDTH, i))
    }

    Ok(())
}

// TODO use result instead of option and handle UTF and parse errors adequately
fn parse_format_number(buf: &[u8], i: usize, j: usize) -> Result<usize, ParseError<'static>> {
    // if i == j { return Err(ParseError::new("begin and start indeces must not be equal", i)) }
    if i > j { return Err(ParseError::new("begin index must smaller than end index", i)) }
    let bytes = unsafe{ slice::from_raw_parts(buf.as_ptr() as *const u8, buf.len()) };
    let utf_str = str::from_utf8(&bytes[i..j])?;
    Ok(utf_str.parse()?)
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, io::stdin};

    use super::*;

    #[test]
    fn test_parse_sexagesimal() {
        assert_eq!(parse_sexagesimal("10:30:18"), parse_sexagesimal("10 30.3"));
        assert_eq!(parse_sexagesimal("10:30:18"), parse_sexagesimal("10.505"));

        assert_eq!(parse_sexagesimal("-10:30:18"), parse_sexagesimal("-10 30.3"));
        assert_eq!(parse_sexagesimal("-10:30:18.0"), parse_sexagesimal("-10 30.3"));
        assert_eq!(parse_sexagesimal("-10:30:18.00"), parse_sexagesimal("-10 30.3"));
        assert_eq!(parse_sexagesimal("-10:30:18"), parse_sexagesimal("-10;30.30"));
        assert_eq!(parse_sexagesimal("-10:30:18"), parse_sexagesimal("-10.505"));

        let nbr = parse_sexagesimal("-10:30:18").unwrap();
        assert_eq!(nbr, -10.505f64);
        assert_eq!(format_sexagesimal(&"%9.6m".parse().unwrap(), nbr), "-10:30:18");
    }

    #[test]
    fn test_format_flags() {
        assert_eq!(FormatFlags::SPACE, 4u8);
        assert_eq!(FormatFlags::SPACE | FormatFlags::MINUS, 5u8);

        let minus_space = FormatFlags(5);

        assert_eq!(minus_space.0 & FormatFlags::SPACE, 4u8);
        assert_eq!(minus_space.0 & FormatFlags::MINUS, 1u8);
        assert_eq!(minus_space.0 & FormatFlags::HASH, 0u8);

        assert!(minus_space.space_flag());
        assert!(minus_space.minus_flag());
        assert!(!minus_space.hash_flag());
        assert!(!minus_space.plus_flag());
        assert!(!minus_space.zero_flag());
    }

    fn assert_format_error(fmt: &str, msg: &str, index: usize) {
        let actual = NumberFormat::try_from(fmt).expect_err("expected format parsing to fail");
        let expected = ParseError::new(msg, index);
        assert_eq!(actual, expected);
    }

    fn assert_format(
        fmt: &str,
        f: Option<FormatFlags>,
        w: Option<usize>,
        p: Option<usize>,
        le: Option<FormatLength>,
        s: char
    ) {
        let fmt = NumberFormat::try_from(fmt).unwrap();
        assert_eq!(fmt.f, f);
        assert_eq!(fmt.w, w);
        assert_eq!(fmt.p, p);
        assert_eq!(fmt.l, le);
        assert_eq!(fmt.s, s);
    }

    #[test]
    fn test_number_format() {
        assert_format("%i", None, None, None, None, 'i');
        assert_format("%d", None, None, None, None, 'd');
        assert_format("%5i", None, Some(5), None, None, 'i');
        assert_format("%10d", None, Some(10), None, None, 'd');
        assert_format("%10f", None, Some(10), None, None, 'f');
        assert_format("%10F", None, Some(10), None, None, 'F');
        assert_format("%10g", None, Some(10), None, None, 'g');
        assert_format("%10G", None, Some(10), None, None, 'G');
        assert_format("%1.2f", None, Some(1), Some(2), None, 'f');
        assert_format("%10.2f", None, Some(10), Some(2), None, 'f');
        assert_format("%10.2F", None, Some(10), Some(2), None, 'F');
        assert_format("%10.2g", None, Some(10), Some(2), None, 'g');
        assert_format("%10.2G", None, Some(10), Some(2), None, 'G');

        // zero flag
        assert_format("%0f", Some(FormatFlags(FormatFlags::ZERO)), None, None, None, 'f');
        assert_format("%01f", Some(FormatFlags(FormatFlags::ZERO)), Some(1), None, None, 'f');
        assert_format("%010f", Some(FormatFlags(FormatFlags::ZERO)), Some(10), None, None, 'f');
        assert_format("%010.2f", Some(FormatFlags(FormatFlags::ZERO)), Some(10), Some(2), None, 'f');
        assert_format("%0.0f", Some(FormatFlags(FormatFlags::ZERO)), None, Some(0), None, 'f');
        assert_format("%0.2f", Some(FormatFlags(FormatFlags::ZERO)), None, Some(2), None, 'f');
        assert_format("%010.f", Some(FormatFlags(FormatFlags::ZERO)), Some(10), None, None, 'f');

        // space flag
        assert_format("% 10i", Some(FormatFlags(FormatFlags::SPACE)), Some(10), None, None, 'i');
        // minus flag
        assert_format("%-10d", Some(FormatFlags(FormatFlags::MINUS)), Some(10), None, None, 'd');
        // plus flag
        assert_format("%+10x", Some(FormatFlags(FormatFlags::PLUS)), Some(10), None, None, 'x');
        // hash flag
        assert_format("%#10X", Some(FormatFlags(FormatFlags::HASH)), Some(10), None, None, 'X');

        let zero_plus = Some(FormatFlags(FormatFlags::ZERO|FormatFlags::PLUS));
        assert_format("%+010f", zero_plus, Some(10), None, None, 'f'); // plus zero
        assert_format("%0+10f", zero_plus, Some(10), None, None, 'f'); // zero plus

        assert_format("%10.0f", None, Some(10), Some(0), None, 'f');
        assert_format("%10.f", None, Some(10), None, None, 'f');

        assert_format_error("%c", FMT_ERR_UNSUPORTED_PRINTF_FORMAT, 1);
        assert_format_error("%s", FMT_ERR_UNSUPORTED_PRINTF_FORMAT, 1);
        assert_format_error("%p", FMT_ERR_UNSUPORTED_PRINTF_FORMAT, 1);
        assert_format_error("%n", FMT_ERR_UNSUPORTED_PRINTF_FORMAT, 1);
        assert_format_error("%%", FMT_ERR_UNSUPORTED_PRINTF_FORMAT, 1);
        assert_format_error("%q", FMT_ERR_UNKNOWN_FORMAT_SPECIFIER, 1);
        assert_format_error("%t", FMT_ERR_NO_FORMAT_SPECIFIER, 2);
        assert_format_error("%10\0.2g", FMT_ERR_PREMATURE_END_OF_FORMAT_STRING, 3);
        assert_format_error("%c", FMT_ERR_UNSUPORTED_PRINTF_FORMAT, 1);
        assert_format_error("%2a2c", FMT_ERR_TRAILING_CHARACTERS_AFTER_FORMAT, 3);
    }

    #[test]
    fn test_sexagesimal_format() {
        // assert_format("%12.9m", None, Some(12), Some(9), None, 'm');
        assert_format("%12.3m", None, Some(12), Some(3), None, 'm');
        assert_format("%12.5m", None, Some(12), Some(5), None, 'm');
        assert_format("%12.6m", None, Some(12), Some(6), None, 'm');
        assert_format("%12.8m", None, Some(12), Some(8), None, 'm');
        assert_format("%12.9m", None, Some(12), Some(9), None, 'm');
        assert_format("%16.6m", None, Some(16), Some(6), None, 'm');
        assert_format("%5.3m", None, Some(5), Some(3), None, 'm');
        assert_format("%7.6m", None, Some(7), Some(6), None, 'm');

        assert_format_error("%12.1m", FMT_ERR_SEXAGESIMAL_FRACTION_MUST_BE_3_5_6_8_OR_9, 5);
        assert_format_error("%12.2m", FMT_ERR_SEXAGESIMAL_FRACTION_MUST_BE_3_5_6_8_OR_9, 5);
        assert_format_error("%12.4m", FMT_ERR_SEXAGESIMAL_FRACTION_MUST_BE_3_5_6_8_OR_9, 5);
        assert_format_error("%12.7m", FMT_ERR_SEXAGESIMAL_FRACTION_MUST_BE_3_5_6_8_OR_9, 5);

        // assert_format_error("%6.6m", FMT_ERR_FORMAT_MUST_BE_ZERO_TERMINATED, 5);
        assert_format_error("%1.9m", FMT_ERR_SEXAGESIMAL_WIDTH_MUST_EQUAL_OR_EXCEED_PRECISION, 4);

        assert_format_error("%.m", FMT_ERR_SEXAGESIMAL_FORMAT_MUST_SPECIFY_WIDTH, 1);
        assert_format_error("%.9m", FMT_ERR_SEXAGESIMAL_FORMAT_MUST_SPECIFY_WIDTH, 1);
        assert_format_error("%12.m", FMT_ERR_SEXAGESIMAL_FORMAT_MUST_SPECIFY_PRECISION, 4);

        assert_format_error("%12m", FMT_ERR_SEXAGESIMAL_FORMAT_MUST_SPECIFY_PRECISION, 3);

        assert_format_error("12.3m", FMT_ERR_FIRST_CHARACTER_MUST_BE_A_PERCENT_SIGN, 0);
        assert_format_error("%12.3", FMT_ERR_NO_FORMAT_SPECIFIER, 5);
    }
}