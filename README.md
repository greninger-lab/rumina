<p align="center">
    <img src="imgs/RUMINA_LOGO_light.png#gh-light-mode-only" width=50% height=50%>
    <img src="imgs/RUMINA_LOGO_dark.png#gh-dark-mode-only" width=50% height=50%>
</p>

## RUMINA - Rust-based Unique Molecular Identifier Network Analysis

RUMINA is a performant pipeline for error correction of Next-Generation Sequencing (NGS) data using Unique Molecular Identifiers (UMIs), fit for shotgun-based and amplicon-based methodologies. 

RUMINA deduplicates reads associated with a single template molecule, using the coordinates of read alignment and UMI barcode sequence to perform correction of PCR and sequencing errors. The above strategy also allows for correction of errors in UMI sequences (directional clustering). 

This pipeline is tested for processing ~600 million reads in ~5 hours, at a rate of 120 million reads processed per hour (tested on 10-core M1 Max Mac Studio).


### Workflow:

<p align="center">
    <img src="imgs/workflow_light.png#gh-light-mode-only" width=100% height=100%>
    <img src="imgs/workflow_dark.png#gh-dark-mode-only" width=100% height=100%>
</p>


### Dependencies: 
- :snake: python ≥ 3.12
- :crab: cargo (rust) ≥ 1.77.0


### Installation:

1. Clone this repository
2. `cd` into cloned repo
3. run `sh install.sh`

This will compile the rust components of the pipeline, set up a python virtual environment with the necessary packages, and create a script named `rumina` to enable running RUMINA from any directory. This script will be located in `$HOME/.cargo/bin/`


### Usage: 
```rumina <input (file or directory)> --grouping_method <grouping_method> --separator <separator> <optional args>```

an example command:<br>
```rumina example.bam --grouping_method directional --separator : --split_window 100```

---
The `input` to `rumina` can be a file or a directory; if a directory, all BAM files within (exlcuding pipeline products) are processed.


###  Arguments 
:small_blue_diamond: = mandatory, no default

##### `input` :small_blue_diamond:
The input file or directory. If a file, it must be: 

- in BAM format
    - The UMI should be in the read QNAME field (see image under `--separator`). Illumina data base-called by [BCL Convert](https://www.illumina.com/products/by-type/informatics-products/basespace-sequence-hub/apps/bcl-convert.html) should be formatted this way by default.

BAM indexes or any files associated with reference genomes are not required.

If the input is a directory, all BAM files within (excluding pipeline products) will be processed per the other arguments specified. 

##### `--grouping_method` :small_blue_diamond:

Specifies how/if to merge UMIs based on edit distance, to account for PCR mutations and NGS errors in UMI sequence. Options are: 
* **directional**: Merge UMIs via directional clustering. See *Amplicon* section for more details. This is the best option for amplicon sequencing data.
* **raw**: Treat each UMI as genuine; UMIs are not merged. This is the best option for metagenomics/shotgun sequencing data.

##### `--separator` :small_blue_diamond:
Specifies the character in the read QNAME delimiting the UMI barcode from the rest of the string. This is usually `_` or `:`.<br>

<p align="center">
    <img src="imgs/barcode_light.png#gh-light-mode-only" width=75% height=75%>
    <img src="imgs/barcode_dark.png#gh-dark-mode-only" width=75% height=75%>
</p>


##### `--split_window` (default = auto)
dictates how to split input BAM files into subfiles (for avoiding memory overflow). <br><br> This is usually necessary for BAM files containing above ~15 million reads, assuming 16GB of total system memory, and has been used to process BAM files containing up to 110 million reads. <br> 

Splitting happens along coordinates of the reference genome in the specified BAM file; If `--split_window 100` was used, reads for every 100bp stretch of the reference would be stored in a separate subfile. These subfiles would be processed individually and merged together into the final output file. Once the final file has been created, the subfiles are deleted.

Because reads are grouped per reference coordinate regardless of splitting, this option does not change underlying analysis.

Options are: 
* **auto**: calculate the recommended subfile size (in basepairs along genome) based on input file size. If `input` is a directory, this will be applied independently to each file within the directory
* **positive integer from 1 - 500**: split input files by a fixed window size. If `input` is a directory, this will be applied to each file within the directory. 
* **none** (default): process the input as one file without splitting into subfiles. If your system has ~16GB of RAM, this is suitable for BAMs containing up to ~15 million reads. Using this option with larger BAMs may result in memory overuse.

##### `--no_report` (optional)

if used, disables depth, coverage, and clustering reporting on output files. This can save up to several minutes per file when working with large BAM files.

reports describe coverage and depth of output files using `bedtools genomecov` via `pybedtools`. UMI groups with the least and most reads, respectively, as well as the number of UMI groups surviving initial filtering, are also recorded. 

#### Todo

- establish UMI clustering methods for shotgun sequencing methods

