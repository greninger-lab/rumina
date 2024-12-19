use indexmap::IndexMap;
use parking_lot::Mutex;
use rayon::prelude::*;
use rust_htslib::bam::{ext::BamRecordExtensions, Record};
use std::collections::HashMap;
use std::str;
use std::sync::Arc;

use crate::merge_report::*;
use crate::realign::{align_to_ref, ReMapper};

pub fn handle_dupes(
    umis_reads: &mut HashMap<String, Vec<Record>>,
    mapper: ReMapper,
    ref_fasta: Vec<u8>,
    min_overlap_bp: usize,
) -> (MergeReport, Vec<Record>) {
    let corrected_reads: Arc<Mutex<Vec<Record>>> = Arc::new(Mutex::new(Vec::new()));
    let results: Arc<Mutex<Vec<MergeResult>>> = Arc::new(Mutex::new(Vec::new()));

    let mut merge_report = MergeReport::new();

    // for (_umi, mut reads) in umis_reads.drain() {
    umis_reads.par_drain().for_each(|(_umi, mut reads)| {
        match reads.len() {
            1 => {
                print!("\rWarning: 1 read found for UMI marked as duplicate. Rerunning RUMINA on this file is recommended. If this issue persists, see GitHub issues page.");
                corrected_reads.lock().extend(reads.drain(..));
            }
            _ => {

                let mut outreads: Vec<Record> = Vec::with_capacity(100);
                let mut merge_results: Vec<MergeResult> = Vec::with_capacity(50);

                // sort the read list by reads likely to overlap
                reads.sort_by(|ra, rb| ra.pos().cmp(&rb.pos())
                    .then_with(|| ra.qname().cmp(&rb.qname()))
                    .then_with(|| ra.is_reverse().cmp(&!rb.is_reverse()))
                    .then_with(|| ra.tid().cmp(&rb.tid()))
                    );

                while !reads.is_empty() {
                    let read = reads.remove(0);

                    let result = find_merges(&read, &mut reads, min_overlap_bp);

                    match result {
                        MergeResult::Discordant(_) => {
                            merge_results.push(result);
                        }

                        MergeResult::NoMerge(_) => {
                            outreads.push(read);
                            merge_results.push(result);
                        }
                        MergeResult::Merge(merged_bases) => {
                            let (start_pos, merged_seq) = construct_sequence(merged_bases.unwrap());
                            let merged_read = construct_read(&read, merged_seq, &mut mapper.clone(), &ref_fasta);
                            outreads.push(merged_read);
                            merge_results.push(MergeResult::Merge(None));
                        }
                    }

                }
                corrected_reads.lock().extend(outreads);
                results.lock().extend(merge_results);
            }
        }
    });

    for res in results.lock().drain(..) {
        merge_report.count(res);
    }

    (
        merge_report,
        Arc::try_unwrap(corrected_reads)
            .expect("Failed to dereference merged reads!")
            .into_inner(),
    )
}

pub fn is_opp_orientation(read_a: &Record, read_b: &Record) -> bool {
    (read_a.is_reverse() && !read_b.is_reverse()) | (!read_a.is_reverse() && read_b.is_reverse())
}

pub fn is_overlap(read_a: &Record, read_b: &Record) -> bool {
    let (ras, rae) = (read_a.reference_start(), read_a.reference_end());
    let (rbs, rbe) = (read_b.reference_start(), read_b.reference_end());

    if rae == rbe && ras == rbs {
        return true;
    }

    ras < rbs && rae >= rbs
}

// for groups of >2 reads, find every overlapping f/r read pair, attempt merge
pub fn find_merges(read: &Record, reads: &mut Vec<Record>, min_overlap_bp: usize) -> MergeResult {
    for (i, other_read) in reads.iter().enumerate() {
        if is_opp_orientation(&read, other_read) && is_overlap(&read, other_read) {
            let merge_result = attempt_merge(&read, other_read, min_overlap_bp);

            match merge_result {
                MergeResult::Discordant(_) => {
                    reads.remove(i);
                }
                MergeResult::NoMerge(_) => {}
                MergeResult::Merge(_) => {
                    reads.remove(i);
                }
            }
            return merge_result;
        }
    }

    MergeResult::NoMerge(None)
}

pub fn construct_sequence<'a>(mut read_blueprint: IndexMap<i64, u8>) -> (i64, Vec<u8>) {
    let mut new_seq = Vec::new();

    read_blueprint.sort_unstable_keys();

    let start = read_blueprint
        .keys()
        .min()
        .expect("unable to find minimum genome pos");

    for base in read_blueprint.values() {
        new_seq.push(*base);
    }

    (*start, new_seq)
}

pub fn construct_read(
    original_read: &Record,
    new_seq: Vec<u8>,
    mapper: &mut ReMapper,
    ref_seq: &Vec<u8>,
) -> Record {
    let mut new_rec = original_read.clone();

    let (start, _end, cigar) = align_to_ref(mapper, &new_seq, &ref_seq);

    let qname = [new_rec.qname(), b":MERGED"].concat();
    new_rec.set(
        &qname,
        // Some(&CigarString(vec![Cigar::Match(new_seq.len() as u32)])),
        Some(&cigar),
        new_seq.as_slice(),
        vec![255; new_seq.len() as usize].as_slice(),
    );

    new_rec.set_pos(start as i64);
    return new_rec;
}

// with two overlapping reads, attempt to merge the reads
// halt if the reads have discordant sequence
pub fn attempt_merge(read_a: &Record, read_b: &Record, min_overlap_bp: usize) -> MergeResult {
    // check that these reads have opposing orientation
    let mut ra: IndexMap<i64, u8> = IndexMap::new();
    let mut rb: IndexMap<i64, u8> = IndexMap::new();

    let ras = read_a.seq().as_bytes();
    let rbs = read_b.seq().as_bytes();

    read_a.aligned_pairs().for_each(|pair| {
        ra.entry(pair[1]).or_insert(ras[pair[0] as usize]);
    });

    read_b.aligned_pairs().for_each(|pair| {
        rb.entry(pair[1]).or_insert(rbs[pair[0] as usize]);
    });

    let mut num_overlap = 0;
    let mut discordant = false;

    for (gpos, nuc) in ra {
        if let Some(other_nuc) = rb.get(&gpos) {
            if *other_nuc != nuc {
                print!(
                    "\rDiscordant read detected! base a: {}, base b: {}",
                    str::from_utf8(&[nuc]).unwrap(),
                    str::from_utf8(&[*other_nuc]).unwrap(),
                );
                discordant = true;
                break;
            } else {
                num_overlap += 1;
            }
        } else {
            rb.entry(gpos).or_insert(nuc);
        }
    }
    if !discordant {
        if num_overlap >= min_overlap_bp {
            MergeResult::Merge(Some(rb))
        } else {
            MergeResult::NoMerge(None)
        }
    } else {
        MergeResult::Discordant(None)
    }
}
