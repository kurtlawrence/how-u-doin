use super::*;

impl Report {
    pub fn chg_set(&self, other: &Self) -> ReportChgs {
        ReportChgs {
            label: self.label != other.label,
            desc: self.desc != other.desc,
            len: self.len != other.len,
            pos: self.pos != other.pos,
            bytes: self.bytes != other.bytes,
            history: self.history != other.history,
        }
    }
}
