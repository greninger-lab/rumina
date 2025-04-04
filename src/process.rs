use crate::args::Args;
use crate::main_dedup::{init_processor, process_chunks};
use crate::pair_merger::PairMerger;
use crate::utils::index_bam;
use crate::utils::{gen_outfile_name, get_file_ext};
use crate::GroupReport;
use colored::Colorize;
use log::{error, info};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs::{read_dir, remove_file};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;
use std::sync::Arc;

pub fn gather_files(input_file: &str) -> HashMap<String, String> {
    let inpath = Path::new(input_file);

    if inpath.is_dir() {
        read_dir(inpath)
            .into_iter()
            .flatten()
            .filter_map(|entry| {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        error!("Unable to read file: {}. Won't be used in processing.", e);
                        return None;
                    }
                };
                let path = entry.path();
                if !path.is_dir() && get_file_ext(&path) == Some("bam") {
                    Some((
                        path.to_string_lossy().into_owned(),
                        entry.file_name().to_string_lossy().into_owned(),
                    ))
                } else {
                    None
                }
            })
            .collect()
    } else {
        std::iter::once((
            inpath.to_string_lossy().into_owned(),
            inpath
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_default(),
        ))
        .collect()
    }
}

pub fn process(input_file: (String, String), args: &Args) {
    let output_file = gen_outfile_name(Some(&args.outdir), "RUMINA", &input_file.1);

    let mut hasher = DefaultHasher::new();
    input_file.hash(&mut hasher);
    let seed = hasher.finish();

    // create deduplication report
    let min_maxes: Arc<Mutex<GroupReport>> = Arc::new(Mutex::new(GroupReport::new()));

    let (bam_reader, pair_reader, bam_writer, mut read_handler) = init_processor(
        input_file.0.clone(),
        output_file.to_string(),
        args.grouping_method.clone(),
        args.threads,
        args.strict_threads,
        args.split_window,
        args.length,
        args.only_group,
        args.singletons,
        args.r1_only,
        min_maxes.clone(),
        seed,
    );

    info!("{:?}", read_handler);

    // holds filtered reads awaiting writing to output bam file
    // do grouping and processing
    process_chunks(
        &mut read_handler,
        bam_reader,
        pair_reader,
        &args.separator,
        bam_writer,
    );
    let num_reads_in = read_handler.read_counter;

    drop(read_handler);

    // do final report
    let mut group_report = Arc::try_unwrap(min_maxes).unwrap().into_inner();
    group_report.num_reads_input_file = num_reads_in;

    // report on min and max number of reads per group
    // this creates minmax.txt
    if !group_report.is_blank() {
        println!("{}", "DONE".green());

        group_report.write_to_report_file(&output_file);
        println!("{}", group_report);
    }

    let idx = index_bam(&output_file, args.threads).expect("Failed to index bam");

    if let Some(ref ref_fasta) = args.merge_pairs {
        let mut merger = PairMerger {
            ref_fasta: ref_fasta.to_string(),
            min_overlap_bp: args.min_overlap_bp,
            threads: args.threads,
            infile: output_file.to_string(),
            outfile: gen_outfile_name(None, "MERGED", &output_file),
            split_window: args.split_window,
        };

        info!("{:?}", merger);

        let merge_report = merger.merge_windows();
        remove_file(output_file).ok();
        remove_file(idx).ok();
        index_bam(&merger.outfile, args.threads).unwrap();
        print!("{merge_report}");
    }
}
