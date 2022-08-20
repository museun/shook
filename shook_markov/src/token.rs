use crate::Word;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Token {
    Word(Word), // TODO a thin box
    End,
}
