use std::fmt;

#[derive(Debug, Clone)]
pub struct Pair {
    pub lh_string: String,
    pub rh_string: String,
    pub lh_start_byte: usize,
    pub lh_end_byte: usize,
    pub lh_start_col: usize,
    pub lh_width: usize,
    pub rh_start_byte: usize,
    pub rh_end_byte: usize,
}

impl fmt::Display for Pair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "lh_string: {}\n rh_string: {}\n lh_start_byte: {}\n lh_end_byte: {}\n lh_start_col: {}\n rh_start_byte: {}\n rh_end_byte: {}\n lh_width: {}\n",
               self.lh_string,
               self.rh_string,
               self.lh_start_byte,
               self.lh_end_byte,
               self.lh_start_col,
               self.rh_start_byte,
               self.rh_end_byte,
               self.lh_width)
    }
}


#[derive(Debug, Clone)]
pub struct Row {
    pub text: String,
    pub start_byte: usize,
    pub end_byte: usize,
}


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AlignKind {
    LetLike,
    MapLike,
    NsLike,
    VecLike,
    CondLike,
    CondPLike,
    ThreadLike,
}

impl fmt::Display for AlignKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlignKind::LetLike => write!(f, "LetLike"),
            AlignKind::MapLike => write!(f, "MapLike"),
            AlignKind::CondLike => write!(f, "CondLike"),
            AlignKind::CondPLike => write!(f, "CondPLike"),
            AlignKind::VecLike => write!(f, "VecLike"),
            AlignKind::NsLike => write!(f, "NsLike"),
            AlignKind::ThreadLike => write!(f, "ThreadLike"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Extracted {
    Pairs(Vec<Pair>),
    Rows {
        anchor_byte: usize,
        rows: Vec<Row>,
    },
}
