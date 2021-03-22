use argparse::{ArgumentParser, Store, List, StoreTrue};
use std::path::Path;

fn main() -> Result<(), String> {
    let mut input = String::new();
    let mut delimiter = String::new();
    let mut only_delimited = false;
    let mut complement = false;
    let mut fields : Vec<String> = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("formats a table-like stream into a LaTeX-table");
        ap.refer(&mut delimiter).add_option(&["-d", "--delimiter"], Store, "use this character for field delimiter").required();
        ap.refer(&mut fields).add_option(&["-f"], List, "select only these fields;  also print any line that contains no delimiter character, unless the -s option is specified").required();
        ap.refer(&mut only_delimited).add_option(&["-s", "--only-delimited"], StoreTrue, "do not print lines not containing delimiters");
        ap.refer(&mut complement).add_option(&["--complement"], StoreTrue, "complement the set of selected bytes, characters or fields");
        ap.refer(&mut input).add_argument("FILE", Store, "name of the input file, or '-' to read from STDIN");
        ap.parse_args_or_exit();
        
    }
    if input.len() < 1 {
        input = String::from("-");
    } else {
        let p = Path::new(&input);
        if ! p.exists() { return Err(format!("{} does not exist", &input)); }
        if ! p.is_file() { return Err(format!("{} is not a file", &input)); }
    }
    println!("input: {}", &input);

    Ok(())
}
