const SQUOTE: u8 = 1;
const DQUOTE: u8 = 2;
const BACKSL: u8 = 4;
const COMM_1: u8 = 8;

pub struct StatementSplitter<'a> {
    data:  &'a str,
    read:  usize,
    flags: u8,
}

impl<'a> StatementSplitter<'a> {
    pub fn new(data: &'a str) -> StatementSplitter<'a> {
        StatementSplitter { data: data, read: 0, flags: 0 }
    }
}

impl<'a> Iterator for StatementSplitter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        let start = self.read;
        let mut levels = 0u8;
        for character in self.data.bytes().skip(self.read) {
            self.read += 1;
            match character {
                _ if self.flags & BACKSL != 0                => self.flags ^= BACKSL,
                b'\'' if self.flags & DQUOTE == 0            => self.flags ^= SQUOTE,
                b'"'  if self.flags & SQUOTE == 0            => self.flags ^= DQUOTE,
                b'\\' if self.flags & (SQUOTE + DQUOTE) == 0 => self.flags |= BACKSL,
                b'$'  if self.flags & (SQUOTE + DQUOTE) == 0 => { self.flags |= COMM_1; continue },
                b'('  if self.flags & COMM_1 != 0            => levels += 1,
                b')'  if levels > 0                          => levels -= 1,
                b';'  if (self.flags & (SQUOTE + DQUOTE) == 0) && levels == 0 => {
                    return Some(self.data[start..self.read-1].trim())
                },
                b'#' if (self.flags & (SQUOTE + DQUOTE) == 0) && levels == 0 => {
                    let output = self.data[start..self.read-1].trim();
                    self.read = self.data.len();
                    return Some(output);
                },
                _ => ()
            }
            self.flags &= 255 ^ COMM_1;
        }

        if start == self.read {
            None
        } else {
            self.read = self.data.len();
            Some(self.data[start..].trim())
        }
    }
}

#[test]
fn statements_with_processes() {
    let command = "echo $(seq 1 10); echo $(seq 1 10)";
    for statement in StatementSplitter::new(command) {
        assert_eq!(statement, "echo $(seq 1 10)");
    }
}

#[test]
fn statements_process_with_statements() {
    let command = "echo $(seq 1 10; seq 1 10)";
    for statement in StatementSplitter::new(command) {
        assert_eq!(statement, command);
    }
}

#[test]
fn statements_with_quotes() {
    let command = "echo \"This ;'is a test\"; echo 'This ;\" is also a test'";
    let results = StatementSplitter::new(command).collect::<Vec<&str>>();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0], "echo \"This ;'is a test\"");
    assert_eq!(results[1], "echo 'This ;\" is also a test'");
}

#[test]
fn statements_with_comments() {
    let command = "echo $(echo one # two); echo three # four";
    let results = StatementSplitter::new(command).collect::<Vec<&str>>();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0], "echo $(echo one # two)");
    assert_eq!(results[1], "echo three");
}

#[test]
fn statements_with_process_recursion() {
    let command = "echo $(echo one $(echo two) three)";
    let results = StatementSplitter::new(command).collect::<Vec<&str>>();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], command);

    let command = "echo $(echo $(echo one; echo two); echo two)";
    let results = StatementSplitter::new(command).collect::<Vec<&str>>();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], command);
}