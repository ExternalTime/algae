use std::borrow::Cow;
use std::collections::{hash_map::Entry, HashMap};

#[derive(Clone, Copy, Debug)]
pub struct Position {
    line: usize,
    offset: usize,
}

struct Parser<'a> {
    str: &'a str,
    pos: Position,
}

#[derive(Debug)]
pub struct ParseError<'a> {
    str: &'a str,
    pos: Position,
    message: std::borrow::Cow<'static, str>,
}

impl<'a> std::fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.message)?;
        let start = self.str[..self.pos.offset]
            .rfind('\n')
            .map(|x| x + 1)
            .unwrap_or(0);
        let end = self.str[self.pos.offset..]
            .find('\n')
            .map(|x| self.pos.offset + x)
            .unwrap_or(self.str.len());
        writeln!(f, "{} | {}", self.pos.line + 1, &self.str[start..end])?;
        for _ in 0..self.pos.line.checked_ilog10().unwrap_or(0) + 1 + 3 {
            write!(f, " ")?;
        }
        for char in self.str[start..self.pos.offset].chars() {
            match char.is_whitespace() {
                true => write!(f, "{char}")?,
                false => write!(f, " ")?,
            }
        }
        writeln!(f, "^")
    }
}

impl<'a> std::error::Error for ParseError<'a> {}

type Result<'a, T> = std::result::Result<T, ParseError<'a>>;
impl<'a> Parser<'a> {
    fn err<E: Into<Cow<'static, str>>>(&self, message: E) -> ParseError<'a> {
        ParseError {
            str: self.str,
            pos: self.pos,
            message: message.into(),
        }
    }

    fn rem(&self) -> &'a str {
        &self.str[self.pos.offset..]
    }

    fn next<T>(&mut self, f: impl FnOnce(&mut Self, char) -> Result<'a, T>) -> Result<'a, T> {
        self.rem()
            .chars()
            .next()
            .ok_or_else(|| self.err("unexpected end of file"))
            .and_then(|c| {
                f(self, c).inspect(|_| {
                    self.pos.offset += c.len_utf8();
                    if c == '\n' {
                        self.pos.line += 1;
                    }
                })
            })
    }

    fn skip_whitespace(&mut self) {
        while self
            .next(|s, c| match c.is_ascii_whitespace() {
                true => Ok(()),
                false => Err(s.err("")),
            })
            .is_ok()
        {}
    }

    fn atoken(&mut self, str: &str) -> Result<'a, ()> {
        match self.rem().starts_with(str) {
            true => Ok(self.pos.offset += str.len()),
            false => Err(self.err(format!("expected \"{}\"", str.escape_debug()))),
        }
    }

    fn token(&mut self, str: &str) -> Result<'a, ()> {
        self.skip_whitespace();
        self.atoken(str)
    }

    fn char_array(&mut self) -> Result<'a, Vec<char>> {
        self.token("\"")
            .map_err(|_| self.err("expected a string (enclosed in quotes)"))?;
        let mut chars = Vec::new();
        loop {
            if self.atoken("\"").is_ok() {
                break;
            }
            let escaped = self.atoken("\\").is_ok();
            self.next(|s, c| match (escaped, c) {
                (false, '\n') => Err(s.err("missing closing '\"'")),
                (false, c) | (true, c @ ('\\' | '"')) => {
                    chars.push(c);
                    Ok(false)
                }
                (true, c) => Err(s.err(format!("\\'{c}' is not a valid escape"))),
            })?;
        }
        Ok(chars)
    }

    fn number(&mut self) -> Result<'a, u64> {
        let tmp = self.pos;
        self.skip_whitespace();
        let mut n = 0u64;
        let mut digit = &mut |s: &mut Self, c: char| {
            let d = c
                .to_digit(10)
                .ok_or_else(|| s.err("expected number"))?
                .into();
            n = n
                .checked_mul(10)
                .and_then(|n| n.checked_add(d))
                .ok_or(s.err("number too big!"))?;
            Ok(())
        };
        self.next(&mut digit).inspect_err(|_| self.pos = tmp)?;
        while self.next(&mut digit).is_ok() {}
        Ok(n)
    }

    fn weight(&mut self, map: &mut HashMap<[char; 2], u64>) -> Result<'a, ()> {
        let tmp = self.pos;
        let Entry::Vacant(e) = map.entry(self.char_array()?.try_into().map_err(|v: Vec<_>| {
            self.err(format!("bigram should have 2 characters, not {}", v.len()))
        })?) else {
            self.pos = tmp;
            return Err(self.err("duplicate bigram"));
        };
        self.token(":")?;
        e.insert(self.number()?);
        Ok(())
    }

    fn weights(&mut self) -> Result<'a, HashMap<[char; 2], u64>> {
        let mut map = HashMap::new();
        self.token("{")?;
        if self.token("}").is_ok() {
            return Ok(map);
        }
        loop {
            self.weight(&mut map)?;
            if self.token("}").is_ok() {
                return Ok(map);
            } else if self.token(",").is_err() {
                return Err(self.err("expected '}' or ','"));
            }
        }
    }

    fn parse(&mut self) -> Result<'a, (Option<[char; 30]>, HashMap<[char; 2], u64>)> {
        self.token("{")?;
        let mut alphabet = None;
        if self.token("\"alphabet\"").is_ok() {
            self.token(":")?;
            alphabet = self
                .char_array()?
                .try_into()
                .map(Some)
                .map_err(|v: Vec<_>| {
                    self.err(format!(
                        "alphabet should have 30 characters, not {}",
                        v.len()
                    ))
                })?;
            self.token(",")?;
        }
        self.token("\"bigrams\"")?;
        self.token(":")?;
        let weights = self.weights()?;
        self.token("}")?;
        self.skip_whitespace();
        match self.rem().is_empty() {
            true => Ok((alphabet, weights)),
            false => Err(self.err("expected end of file")),
        }
    }
}

pub fn parse<'a>(
    str: &'a str,
) -> std::result::Result<(Option<[char; 30]>, HashMap<[char; 2], u64>), impl std::fmt::Display> {
    let mut parser = Parser {
        str,
        pos: Position { line: 0, offset: 0 },
    };
    parser.parse().map_err(|e| e.to_string())
}
