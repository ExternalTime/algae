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

fn err_to_string(err: Cow<str>, source: &str, pos: Position) -> String {
    let mut message = err.into_owned();
    let start = source[..pos.offset].rfind('\n').map(|x| x + 1).unwrap_or(0);
    let end = source[pos.offset..]
        .find('\n')
        .map(|x| pos.offset + x)
        .unwrap_or(source.len());
    message.push('\n');
    message.push_str(pos.line.to_string().as_str());
    message.push_str(" | ");
    message.push_str(source[start..end].trim_end());
    message.push('\n');
    for _ in 0..pos.line.checked_ilog10().unwrap_or(0) + 1 + 3 {
        message.push(' ');
    }
    for char in source[start..pos.offset].chars() {
        message.push(match char.is_whitespace() {
            true => char,
            false => ' ',
        });
    }
    message.push('^');
    message
}

type Result<T> = std::result::Result<T, Cow<'static, str>>;

impl<'a> Parser<'a> {
    fn rem(&self) -> &'a str {
        &self.str[self.pos.offset..]
    }

    fn peek(&self) -> Option<char> {
        self.rem().chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        self.peek().inspect(|&c| {
            self.pos.offset += c.len_utf8();
            if c == '\n' {
                self.pos.line += 1;
            }
        })
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|c| c.is_ascii_whitespace()) {
            self.advance();
        }
    }

    fn atoken(&mut self, str: &str) -> Result<()> {
        match self.rem().starts_with(str) {
            true => Ok(self.pos.offset += str.len()),
            false => Err(format!("expected \"{}\"", str.escape_debug()).into()),
        }
    }

    fn token(&mut self, str: &str) -> Result<()> {
        self.skip_whitespace();
        self.atoken(str)
    }

    fn unicode_escape(&mut self) -> Result<char> {
        self.advance();
        let mut n = 0u32;
        for _ in 0..4 {
            n = n * 16 + self.advance()
                .and_then(|c| c.to_digit(16))
                .ok_or(Cow::Borrowed("invalid \\u escape"))?;
        }
        char::from_u32(n).ok_or("surrogates not currently supported".into())
    }

    fn char_array(&mut self) -> Result<Vec<char>> {
        self.token("\"")
            .map_err(|_| Cow::Borrowed("expected a string (enclosed in quotes)"))?;
        let mut chars = Vec::new();
        while self.atoken("\"").is_err() {
            let pos = self.pos;
            let escaped = self.atoken("\\").is_ok();
            let c = self.peek().ok_or(Cow::Borrowed("missing closing '\"'"))?;
            match (escaped, c) {
                (false, '\n') => return Err("missing closing '\"'".into()),
                (false, c) | (true, c @ ('\\' | '"')) => {
                    chars.push(c);
                    self.advance();
                }
                (true, 'u') => chars.push(self.unicode_escape().inspect_err(|_| self.pos = pos)?),
                (true, c) => return Err(format!("\\'{c}' is not a valid escape").into()),
            };
        }
        Ok(chars)
    }

    fn number(&mut self) -> Result<u64> {
        self.skip_whitespace();
        let start_pos = self.pos;
        let next_digit = |s: &mut Self| s.peek().and_then(|c| c.to_digit(10).map(u64::from));
        let mut n = next_digit(self).ok_or("expected a number")?;
        self.advance();
        while let Some(d) = next_digit(self) {
            n = n
                .checked_mul(10)
                .and_then(|n| n.checked_add(d))
                .ok_or("number too big")
                .inspect_err(|_| self.pos = start_pos)?;
            self.advance();
        }
        Ok(n)
    }

    fn weight(&mut self, map: &mut HashMap<[char; 2], u64>) -> Result<()> {
        let pos = self.pos;
        let Entry::Vacant(e) = map.entry(self.char_array()?.try_into().map_err(|v: Vec<_>| {
            Cow::Owned(format!("bigram should have 2 characters, not {}", v.len()))
        })?) else {
            self.pos = pos;
            return Err("duplicate bigram".into());
        };
        self.token(":")?;
        e.insert(self.number()?);
        Ok(())
    }

    fn weights(&mut self) -> Result<HashMap<[char; 2], u64>> {
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
                return Err("expected '}' or ','".into());
            }
        }
    }

    fn alphabet(&mut self) -> Result<[char; 30]> {
        self.char_array()?.try_into().map_err(|v: Vec<_>| {
            format!("alphabet should have 30 characters, not {}", v.len()).into()
        })
    }

    fn parse(&mut self) -> Result<(Option<[char; 30]>, HashMap<[char; 2], u64>)> {
        self.token("{")?;
        let pos = self.pos;
        if self.token("}").is_ok() {
            self.pos = pos;
            return Err("weights are not defined".into());
        }
        let mut alphabet = None;
        let mut weights = None;
        loop {
            self.skip_whitespace();
            let pos = self.pos;
            if self.token(r#""alphabet""#).is_ok() {
                if alphabet.is_some() {
                    self.pos = pos;
                    return Err("duplicate alphabet definition".into());
                }
                self.token(":")?;
                alphabet = self.alphabet().map(Some)?;
            } else if self.token(r#""weights""#).is_ok() {
                if weights.is_some() {
                    self.pos = pos;
                    return Err("duplicate weights definition".into());
                }
                self.token(":")?;
                weights = self.weights().map(Some)?;
            } else {
                return Err("expected either either alphabet or weights".into());
            }
            if self.token("}").is_ok() {
                let Some(weights) = weights else {
                    return Err("weights are not defined".into());
                };
                self.skip_whitespace();
                return match self.rem().is_empty() {
                    true => Ok((alphabet, weights)),
                    false => Err("expected end of file".into()),
                };
            } else if self.token(",").is_err() {
                return Err("expected ',' or '}'".into());
            }
        }
    }
}

pub fn parse(
    str: &'_ str,
) -> std::result::Result<(Option<[char; 30]>, HashMap<[char; 2], u64>), impl std::fmt::Display> {
    let mut parser = Parser {
        str,
        pos: Position { line: 1, offset: 0 },
    };
    parser
        .parse()
        .map_err(|e| err_to_string(e, str, parser.pos))
}
