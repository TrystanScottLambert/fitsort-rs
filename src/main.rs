//! fitsort — sort FITS keywords into a table
//!
//! Companion to `dfits`. Reads `dfits` output from stdin, extracts the
//! keyword values named on the command line, and prints one aligned row
//! per file.
//!
//!     dfits *.fits | fitsort BITPIX NAXIS NAXIS1 NAXIS2
//!
//! Values are tab-separated, records newline-separated. A keyword absent
//! from a given file produces an empty column. `-d` suppresses the
//! header row.
//!
//! Rust port of Nicolas Devillard's fitsort.c (1996).

use clap::Parser;
use std::io::{self, BufRead, IsTerminal, Write};
use std::process::ExitCode;

const CARD_BYTES: usize = 80;

#[derive(Parser, Debug)]
#[command(
    name = "fitsort",
    about = "Tabulate FITS keyword values from dfits output",
    long_about = "Reads dfits output on stdin and prints the requested \
                  keyword values as an aligned, tab-separated table.\n\n\
                  Example:\n    dfits *.fits | fitsort BITPIX NAXIS NAXIS1"
)]
struct Cli {
    /// Suppress the header row (column titles).
    #[arg(short = 'd', long = "no-header")]
    no_header: bool,

    /// Keywords to extract. Case-insensitive; dotted names like
    /// `DET.DIT` expand to `HIERARCH ESO DET DIT`.
    #[arg(value_name = "KEYWORD", required = true)]
    keywords: Vec<String>,
}

#[derive(Debug)]
struct Keyword {
    display: String,
    match_form: String,
}

impl Keyword {
    fn parse(raw: &str) -> Self {
        let display = raw.to_uppercase();
        let match_form = if display.contains('.') {
            expand_hierarch(&display)
        } else {
            display.clone()
        };
        Keyword {
            display,
            match_form,
        }
    }
}

/// Expand a dotted keyword `A.B.C` into `HIERARCH ESO A B C`.
fn expand_hierarch(dotted: &str) -> String {
    let mut out = String::from("HIERARCH ESO");
    for part in dotted.split('.') {
        out.push(' ');
        out.push_str(part);
    }
    out
}

#[derive(Debug)]
struct FileRecord {
    filename: Option<String>,
    values: Vec<Option<String>>,
}

impl FileRecord {
    fn new(filename: Option<String>, keyword_count: usize) -> Self {
        FileRecord {
            filename,
            values: vec![None; keyword_count],
        }
    }
}

enum LineKind {
    FileBanner(String),
    SimpleCard,
    HeaderCard,
}

fn classify_line(line: &str) -> LineKind {
    if line.starts_with("====>") {
        let name = line.split_whitespace().nth(2).unwrap_or("").to_string();
        LineKind::FileBanner(name)
    } else if line.starts_with("SIMPLE  =") {
        LineKind::SimpleCard
    } else {
        LineKind::HeaderCard
    }
}

fn match_keyword(line: &str, keywords: &[Keyword]) -> Option<usize> {
    let field = line.split('=').next()?.trim_end();
    keywords.iter().position(|kw| kw.match_form == field)
}

fn extract_value(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let eq = bytes.iter().position(|&b| b == b'=')?;

    // Collect the value region, honouring quote state.
    let mut raw = String::new();
    let mut in_quote = false;
    for &b in bytes
        .iter()
        .skip(eq + 1)
        .take(CARD_BYTES.saturating_sub(eq + 1))
    {
        if b == b'/' && !in_quote {
            break;
        }
        if b == b'\'' {
            in_quote = !in_quote;
        }
        raw.push(b as char);
    }

    if let Some(first) = raw.find('\'') {
        if let Some(last) = raw.rfind('\'') {
            if last > first {
                return Some(raw[first + 1..last].to_string());
            }
        }
    }

    raw.split_whitespace().next().map(str::to_string)
}

fn collect_records<R: BufRead>(
    reader: &mut R,
    keywords: &[Keyword],
) -> io::Result<Vec<FileRecord>> {
    let mut records: Vec<FileRecord> = Vec::new();
    let mut lines = reader.lines();

    while let Some(line) = lines.next() {
        let line = line?;
        match classify_line(&line) {
            LineKind::FileBanner(name) => {
                records.push(FileRecord::new(Some(name), keywords.len()));
                // The banner is immediately followed by the SIMPLE card;
                // consume it so it isn't mistaken for another new file.
                let _ = lines.next();
            }
            LineKind::SimpleCard => {
                records.push(FileRecord::new(None, keywords.len()));
            }
            LineKind::HeaderCard => {
                if let Some(index) = match_keyword(&line, keywords) {
                    if let Some(current) = records.last_mut() {
                        if let Some(value) = extract_value(&line) {
                            current.values[index] = Some(value);
                        }
                    }
                }
            }
        }
    }
    Ok(records)
}

struct Layout {
    filename_width: usize,
    keyword_widths: Vec<usize>,
    show_filenames: bool,
}

fn compute_layout(records: &[FileRecord], keywords: &[Keyword]) -> Layout {
    let mut keyword_widths: Vec<usize> = keywords.iter().map(|kw| kw.display.len()).collect();
    let mut filename_width = 0;
    let mut show_filenames = false;

    for record in records {
        if let Some(name) = &record.filename {
            show_filenames = true;
            filename_width = filename_width.max(name.len());
        }
        for (slot, value) in record.values.iter().enumerate() {
            if let Some(v) = value {
                keyword_widths[slot] = keyword_widths[slot].max(v.len());
            }
        }
    }

    Layout {
        filename_width,
        keyword_widths,
        show_filenames,
    }
}

/// Write the table to `out`.
fn write_table<W: Write>(
    out: &mut W,
    records: &[FileRecord],
    keywords: &[Keyword],
    layout: &Layout,
    print_header: bool,
) -> io::Result<()> {
    if print_header {
        if layout.show_filenames {
            write!(out, "{:<width$}\t", "FILE", width = layout.filename_width)?;
        }
        for (kw, &width) in keywords.iter().zip(&layout.keyword_widths) {
            write!(out, "{:<width$}\t", kw.display, width = width)?;
        }
        writeln!(out)?;
    }

    for record in records {
        if layout.show_filenames {
            let name = record.filename.as_deref().unwrap_or("");
            write!(out, "{:<width$}\t", name, width = layout.filename_width)?;
        }
        for (slot, &width) in layout.keyword_widths.iter().enumerate() {
            let cell = record.values[slot].as_deref().unwrap_or("");
            write!(out, "{:<width$}\t", cell, width = width)?;
        }
        writeln!(out)?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if io::stdin().is_terminal() {
        eprintln!("fitsort: expecting dfits output on stdin (try: dfits *.fits | fitsort ...)");
        return ExitCode::from(1);
    }

    let keywords: Vec<Keyword> = cli.keywords.iter().map(|k| Keyword::parse(k)).collect();

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let records = match collect_records(&mut reader, &keywords) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("fitsort: error reading input: {}", e);
            return ExitCode::from(1);
        }
    };

    if records.is_empty() {
        eprintln!("fitsort: no input data corresponding to dfits output");
        return ExitCode::from(1);
    }

    let layout = compute_layout(&records, &keywords);

    let stdout = io::stdout();
    let mut out = stdout.lock();
    if let Err(e) = write_table(&mut out, &records, &keywords, &layout, !cli.no_header) {
        eprintln!("fitsort: error writing output: {}", e);
        return ExitCode::from(1);
    }

    ExitCode::from(0)
}
