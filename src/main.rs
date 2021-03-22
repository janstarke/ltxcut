use argparse::{ArgumentParser, Store, List, StoreTrue};
use std::path::Path;
use std::io::{BufRead, BufReader};
use std::fs::File;

struct CutOptions {
    input_delimiter: String,
    output_delimiter: String,
    only_delimited: bool,
    complement: bool,
    fields: Vec<String>,
    wrap_lines: String,
    wrap_fields : Vec<String>
}

impl CutOptions {
    pub fn new() -> CutOptions {
        CutOptions {
            input_delimiter: String::new(),
            output_delimiter: String::from(" & "),
            only_delimited: false,
            complement: false,
            fields: Vec::new(),
            wrap_lines: String::new(),
            wrap_fields: Vec::new(),
        }
    }

    fn cut_line(&self, line: &str) -> std::io::Result<()> {
        Ok(())
    }

    fn cut<T: BufRead>(&self, reader: T) -> std::io::Result<()> {
        for line in reader.lines() {
            self.cut_line(&line?)?;
        }
        Ok(())
    }
}

fn main() -> Result<(), String> {
    let mut options = CutOptions::new();
    let mut input = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("formats a table-like stream into a LaTeX-table");
        ap.refer(&mut options.input_delimiter).add_option(&["-d", "--delimiter"], Store, "use this character for field delimiter").required();
        ap.refer(&mut options.fields).add_option(&["-f"], List, "select only these fields;  also print any line that contains no delimiter character, unless the -s option is specified").required();
        ap.refer(&mut options.only_delimited).add_option(&["-s", "--only-delimited"], StoreTrue, "do not print lines not containing delimiters");
        ap.refer(&mut options.complement).add_option(&["--complement"], StoreTrue, "complement the set of selected bytes, characters or fields");
        ap.refer(&mut options.wrap_lines).add_option(&["-l", "--wrap-lines"], Store, "use this LaTeX command to wrap lines");
        ap.refer(&mut options.wrap_fields).add_option(&["-c", "--wrap-columns"], List, "use this LaTeX commands to wrap columns");
        ap.refer(&mut input).add_argument("FILE", Store, "name of the input file, or '-' to read from STDIN");
        ap.parse_args_or_exit();
        
    }
    let res = 
    if input.is_empty() {
        options.cut(std::io::stdin().lock())
    } else {
        let p = Path::new(&input);
        if ! p.exists() { return Err(format!("{} does not exist", &input)); }
        if ! p.is_file() { return Err(format!("{} is not a file", &input)); }
        match File::open(p) {
            Ok(f) => options.cut(BufReader::new(f)),
            Err(why) => Err(why)
        }
    };

    match res {
        Err(why) => Err(why.to_string()),
        Ok(())   => Ok(()),
    }
}
