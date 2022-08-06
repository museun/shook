use super::Tags;

pub fn parse(mut line: &str) -> (Tags, Option<&str>, &str, Vec<&str>, Option<&str>) {
    let line = &mut line;
    let tags = if line.starts_with('@') {
        Tags::parse(line)
    } else {
        None
    }
    .unwrap_or_default();

    let prefix = if line.starts_with(':') {
        prefix(line).map(Into::into)
    } else {
        None
    };

    let command = command(line);
    let args = args(line);
    let data = data(line).map(Into::into);

    (tags, prefix, command, args, data)
}

fn prefix<'a>(input: &mut &'a str) -> Option<&'a str> {
    if input.starts_with(':') {
        let (head, tail) = input.split_once(' ').expect("malformed message");
        *input = tail;
        return head[1..].split_terminator('!').next();
    }
    None
}

fn command<'a>(input: &mut &'a str) -> &'a str {
    // TODO we got a panic ehre
    let (head, tail) = input.split_once(' ').expect("malformed message");
    *input = tail;
    head
}

fn args<'a>(input: &mut &'a str) -> Vec<&'a str> {
    if let Some((head, tail)) = input.split_once(':') {
        *input = tail;
        head.split_ascii_whitespace().collect()
    } else {
        vec![]
    }
}

fn data<'a>(input: &mut &'a str) -> Option<&'a str> {
    Some(input.trim_end()).filter(|s| !s.is_empty())
}
