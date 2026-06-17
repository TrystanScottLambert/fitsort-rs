# fitsort-rs
A rust rewrite of fitsort


![Demo](.github/fitsort.gif)

# Motivation
`fitsort` (and it's sister program `dfits`) are amazingly useful pieces of software for batch processing the headers of many fits files. These were originally built in C by Nicolas Devillard (The original eclipse packages can be found on hit [github page](https://github.com/ndevilla/eclipse)).
However, the main webpage for these tools is beginning to go [stale](https://www.eso.org/sci/software/eclipse/eug/eug/node8.html) with some links no longer working. The fear of loosing these tools is too great. Therefore some effort has gone in to preserve them in some way. Noteably [storing the raw c files](https://github.com/granttremblay/eso_fits_tools) and a [python rewrite](https://github.com/Romain-Thomas-Shef/dfitspy).
This repo, along with [dfits-rs](https://github.com/TrystanScottLambert/dfits-rs) are rust rewrites of both `fitsort` and `dfits` which aims to 1) preserve the functionality of the original. And 2) allow improvements if needed. This current version aims to be a truthful remake of the original matching the output exactly but I don't guarantee this will always be the case (But version 0.1.0 will be).

# Usage
`fitsort` is not used on its own. It reads the header output of [dfits](https://github.com/TrystanScottLambert/dfits-rs) from standard input and tabulates the keywords you ask for, one row per file. You pipe `dfits` into it.
```bash
dfits example.fits | fitsort NAXIS1 NAXIS2
```
will print a table with one column per keyword and the value pulled from `example.fits`. Where `fitsort` becomes powerful is across many files, where it lines everything up into a single readable table.
```bash
dfits *.fits | fitsort BITPIX NAXIS NAXIS1 NAXIS2 NAXIS3
```
The output is a plain table: values are separated by tabs and records by newlines, so it drops straight into other tools or into a file for later.
```bash
dfits *.fits | fitsort EXPTIME OBJECT > observations.tsv
```
Keywords are case-insensitive, so `naxis1` and `NAXIS1` are treated the same. When a keyword is absent from a given file's header that column is simply left blank for that row, so files with different headers can still be compared side by side.
The `-d` flag suppresses the header row (the line of column titles), which is useful when the output is being fed to another program.
```bash
dfits *.fits | fitsort -d NAXIS1 NAXIS2
```
ESO `HIERARCH` keywords are supported. Type them in the compact dotted form and `fitsort` expands `A.B.C` into `HIERARCH ESO A B C` for you.
```bash
dfits *.fits | fitsort DET.DIT DET.NDIT
```

# Install
We provide several easy options for installing `fitsort`.
## Downloading binaries
### Macos
For newer macs (m-series) the following commands should work.
```
curl -L fitsort https://github.com/trystanscottlambert/fitsort-rs/releases/download/v0.1.0/fitsort-aarch64-apple-darwin
chmod +x fitsort-aarch64-apple-darwin
sudo mv fitsort-aarch64-apple-darwin /usr/local/bin/fitsort
```
Alternatively for older macs
```
curl -L fitsort https://github.com/trystanscottlambert/fitsort-rs/releases/download/v0.1.0/fitsort-x86_64-apple-darwin
chmod +x fitsort-x86_64-apple-darwin
sudo mv fitsort-x86_64-apple-darwin /usr/local/bin/fitsort
```
### Linux
For Ubuntu/Debian
```
curl -L fitsort https://github.com/trystanscottlambert/fitsort-rs/releases/download/v0.1.0/fitsort-x86_64-unknown-linux-gnu
chmod +x fitsort-x86_64-unknown-linux-gnu
sudo mv fitsort-x86_64-unknown-linux-gnu /usr/local/bin/fitsort
```
## Building from source
If you don't want to download binaries and prefer to compile everything from source, then this can be done very easily using rust.
If you don't have rust first install it with the following command and follow the prompts.
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Clone the repo and `cd` into it and build the binaries.
```
git clone git@github.com:TrystanScottLambert/fitsort-rs.git
cd fitsort-rs
cargo build --release
```
Move the binary into /usr/local/bin
```
sudo mv target/release/fitsort /usr/local/bin/
```
## Cargo
If you are already comfortable with rust then you can just install `fitsort` using cargo.
```
cargo install fitsort-rs
```
