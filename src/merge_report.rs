use colored::Colorize;
use indexmap::IndexMap;
use num_format::{Locale, ToFormattedString};
use std::fmt;

const LOCALE: Locale = Locale::en;

#[derive(Debug)]
pub enum MergeResult {
    Discordant(()),
    NoMerge(()),
    Merge(Option<IndexMap<i64, u8>>),
}

pub struct MergeReport {
    num_discordant: i32,
    num_unmerged: i32,
    num_merged: i32,
    pub num_inreads: i32,
    pub num_outreads: i32,
}

impl MergeReport {
    pub fn count(&mut self, merge_result: MergeResult) {
        match merge_result {
            MergeResult::Discordant(_) => self.num_discordant += 1,
            MergeResult::NoMerge(_) => self.num_unmerged += 1,
            MergeResult::Merge(_) => self.num_merged += 1,
        }
    }

    pub fn new() -> Self {
        MergeReport {
            num_discordant: 0,
            num_unmerged: 0,
            num_merged: 0,
            num_inreads: 0,
            num_outreads: 0,
        }
    }
}
impl fmt::Debug for MergeReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "\n{}\n\
            {}\n\
            {}: {}\n\
            {}: {}\n\
            {}: {}\n\
            {}: {}\n\
            {}: {}\n\
            {}\n\
            ",
            "PAIR MERGER",
            "=============================",
            "Discordant read pairs",
            self.num_discordant.to_formatted_string(&LOCALE),
            "Unmerged reads",
            self.num_unmerged.to_formatted_string(&LOCALE),
            "Merged read pairs",
            self.num_merged.to_formatted_string(&LOCALE),
            "Reads in",
            self.num_inreads.to_formatted_string(&LOCALE),
            "Reads out",
            self.num_outreads.to_formatted_string(&LOCALE),
            "=============================",
        )
    }
}

impl fmt::Display for MergeReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "\n{}\n\
            {}\n\
            {}: {}\n\
            {}: {}\n\
            {}: {}\n\
            {}: {}\n\
            {}: {}\n\
            {}\n\
            ",
            "PAIR MERGER".yellow(),
            "=============================".yellow(),
            "Discordant read pairs".yellow(),
            self.num_discordant.to_formatted_string(&LOCALE),
            "Unmerged reads".yellow(),
            self.num_unmerged.to_formatted_string(&LOCALE),
            "Merged read pairs".yellow(),
            self.num_merged.to_formatted_string(&LOCALE),
            "Reads in".yellow(),
            self.num_inreads.to_formatted_string(&LOCALE),
            "Reads out".yellow(),
            self.num_outreads.to_formatted_string(&LOCALE),
            "=============================".yellow(),
        )
    }
}
