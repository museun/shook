use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

#[derive(Default, Debug, Clone)]
pub struct Arguments {
    pub map: HashMap<String, String>,
}

impl Arguments {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|s| &**s)
    }

    pub fn get_parsed<T>(&self, key: &str) -> Option<anyhow::Result<T>>
    where
        T: FromStr,
        T::Err: Into<anyhow::Error>,
    {
        self.get(key)
            .map(<str>::parse)
            .map(|c| c.map_err(Into::into))
    }
}

impl std::ops::Index<&str> for Arguments {
    type Output = str;

    fn index(&self, key: &str) -> &Self::Output {
        self.get(key)
            .unwrap_or_else(|| panic!("{key} should exist"))
    }
}

#[derive(Debug)]
pub enum Match<T> {
    Required,
    Match(T),
    NoMatch,
    Exact,
}

#[derive(Default, Debug)]
pub struct ExampleArgs {
    pub usage: Box<str>,
    pub args: Box<[ArgType]>,
}

impl ExampleArgs {
    pub fn contains(&self, arg: &Kind) -> bool {
        self.args.iter().any(|ArgType { kind, .. }| kind == arg)
    }
}

impl ExampleArgs {
    const REQUIRED: Kind = Kind::Required;
    const OPTIONAL: Kind = Kind::Optional;
    const VARIADIC: Kind = Kind::Variadic;

    pub fn extract(&self, mut input: &str) -> Match<HashMap<String, String>> {
        if input.is_empty() {
            if self.contains(&Self::REQUIRED) {
                return Match::Required;
            }
            if !self.args.is_empty()
                && (!self.contains(&Self::OPTIONAL) && !self.contains(&Self::VARIADIC))
            {
                return Match::NoMatch;
            }
            if self.args.is_empty() {
                return Match::Exact;
            }
        }

        if !input.is_empty() && self.args.is_empty() {
            return Match::NoMatch;
        }

        use Kind::*;
        let mut map = HashMap::new();
        for ArgType { key, kind } in &*self.args {
            match (kind, input.find(' ')) {
                (Required | Optional, None) | (Variadic, ..) => {
                    if !input.is_empty() {
                        map.insert(key.into(), input.into());
                    }
                    break;
                }
                (.., Some(pos)) => {
                    let (head, tail) = input.split_at(pos);
                    map.insert(key.into(), head.into());
                    input = tail.trim();
                }
            }
        }

        Match::Match(map)
    }

    pub fn parse(input: &str) -> anyhow::Result<Self> {
        // <required> <optional?> <rest..>
        let mut seen = HashSet::new();
        let mut args = vec![];

        for token in input.split_ascii_whitespace() {
            let mut append = |arg: &[_]| {
                let data = &token[1..arg.len() + 1];
                anyhow::ensure!(seen.insert(data), "{data} was already used");
                Ok(data.into())
            };

            let all_alpha = |s: &[u8]| s.iter().all(u8::is_ascii_alphabetic);

            let arg = match token.as_bytes() {
                [b'<', arg @ .., b'.', b'.', b'>'] if all_alpha(arg) => ArgType {
                    key: append(arg)?,
                    kind: Kind::Variadic,
                },
                [b'<', arg @ .., b'?', b'>'] if all_alpha(arg) => ArgType {
                    key: append(arg)?,
                    kind: Kind::Optional,
                },
                [b'<', arg @ .., b'>'] if all_alpha(arg) => ArgType {
                    key: append(arg)?,
                    kind: Kind::Required,
                },
                // TODO report invalid patterns
                // TODO report invalid characters in keys
                _ => continue,
            };

            args.push(arg);
            if matches!(
                args.last(),
                Some(&ArgType {
                    kind: Kind::Variadic,
                    ..
                })
            ) {
                break;
            }
        }

        Ok(Self {
            usage: Box::from(input),
            args: args.into(),
        })
    }
}

#[derive(Debug)]
pub struct ArgType {
    key: String,
    kind: Kind,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Required,
    Optional,
    Variadic,
}
