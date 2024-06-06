import pysam 
import subprocess 
import os
from cov_reporter import report_coverage, get_counts, generate_report

### import args
from args import init_args
args = init_args()

### Find parent directory of input .bam
work_path = args.input

if os.path.isabs(work_path):
    work_path = os.path.dirname(work_path)
else:
    work_path = os.path.abspath(
        os.path.dirname(args.input))

exec_path = os.path.dirname(os.path.abspath(__file__))

print(f"Working path: {work_path}")

def calculate_split(input):
    size = int(os.stat(input).st_size / 1024**2)

    if size in range(0, 500):
        return 0

    elif size in range(500, 1_000):
        return 250

    elif size in range(1000, 10_000):
        return 100

    else:
        return 100

def split_bam(input, window_size):
    file = os.path.basename(input)
    split_dir = file.split('.bam')[0] + f"_{window_size}"

    print(f"Splitting {input} into {window_size}bp windows...")

    subprocess.run([os.path.join(exec_path, "multibam/target/release/multibam"), input, split_dir, str(window_size)])

    return input, os.path.join(os.path.dirname(input), split_dir)

def merge_processed_splits(file):
    clean_dir = os.path.join(work_path, "cleaned")

    bam_name = os.path.basename(file).split('.bam')[0]

    prefixes_for_merging = set()

    for filename in os.listdir(clean_dir):
        if filename.endswith('.bam') and (bam_name in filename and 'final' not in filename):
            prefixes_for_merging.add(filename.split('.')[0])

    for prefix in prefixes_for_merging:
        splits = [os.path.join(clean_dir, file) for file in os.listdir(clean_dir) if file.startswith(prefix) and file.endswith('.bam') and 'final' not in file]
        final_file = os.path.join(clean_dir, prefix + "_final.bam")
        pysam.merge("-@ 6", "-f", final_file, *splits)
        for split in splits:
            os.remove(split)

        if not args.no_report:
            sort_and_index(final_file)
            generate_report(file, final_file)

### assign UG tag for each group of clustered UMIs
def group_bam(input_file, split):

    output_dir = os.path.join(work_path, 'cleaned')
    if not os.path.exists(output_dir):
        os.mkdir(output_dir)

    tagged_file_name = os.path.join(
        os.path.abspath(output_dir), 
        os.path.basename(input_file).split('.bam')[0] + '_cleaned.bam'
    )
    
    tag_cmd = os.path.join(exec_path, 'bam_processor/target/release/bam_processor')
    subprocess.run([tag_cmd, input_file, tagged_file_name, args.separator, args.grouping_method])

    if not args.no_report:
        if not split:

            sort_and_index(tagged_file_name)
            generate_report(input_file, tagged_file_name)

    return(tagged_file_name)

def sort_and_index(output_file):

    temp_file = output_file.split('.bam')[0] + '_s.bam'
    os.rename(output_file, temp_file)

    pysam.sort('-@ 6', temp_file, '-o', output_file)
    os.remove(temp_file)

    pysam.index('-@ 6', output_file)



