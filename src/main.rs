use argparse::{ArgumentParser, Store, StoreTrue};
use std::path::Path;
use std::io::{BufRead, BufReader};
use std::fs::File;
use string_builder::Builder;

enum EncodingOption {
    LaTeX,
    Listing,
    None
}

struct CutOptions {
    input_delimiter: String,
    output_delimiter: String,
    notonly_delimited: bool,
//    complement: bool,
    fields: Vec<usize>,
    wrap_lines: String,
    wrap_fields : Vec<String>,
    escape_fields: Vec<EncodingOption>
}

impl CutOptions {
    pub fn new() -> CutOptions {
        CutOptions {
            input_delimiter: String::new(),
            output_delimiter: String::from(" & "),
            notonly_delimited: false,
//            complement: false,
            fields: Vec::new(),
            wrap_lines: String::new(),
            wrap_fields: Vec::new(),
            escape_fields: Vec::new(),
        }
    }

    fn encode_field(&self, id: usize, value: &str) -> String {
        if id >= self.escape_fields.len() {
            return String::from(value);
        }
        match self.escape_fields[id] {
            EncodingOption::LaTeX   => escape_latex(value),
            EncodingOption::Listing => escape_listing(value),
            EncodingOption::None    => String::from(value)
        }
    }

    fn wrap_field(&self, id: usize, value: &str) -> String {
        if id >= self.wrap_fields.len() {
            return String::from(value);
        }

        let wrap_function = &self.wrap_fields[id];
        if wrap_function.is_empty() {
            return self.encode_field(id, value);
        }

        String::from(format!("\\{}{{{}}}", &wrap_function, self.encode_field(id, value)))
    }

    fn get_field(&self, id: usize, input_fields: &Vec<&str>) -> String {
        self.wrap_field(id,
            if id < input_fields.len() {
                input_fields[id]
            } else {
                ""
            }
        )
    }

    fn cut_line(&self, line: &str) -> std::io::Result<()> {
        if ! line.contains(&self.input_delimiter) {
            if ! self.notonly_delimited {
                println!("{}", line);
            }
            return Ok(())
        }

        let input_fields: Vec<&str> = line.split(&self.input_delimiter).collect();
        let mut output_fields: Vec<String> = Vec::new();
        for column_id in &self.fields {
            let id = column_id - 1;

            // handle missing field
            if id >= input_fields.len() {
                output_fields.push(String::new());
                continue;
            }

            output_fields.push(self.get_field(id, &input_fields));
        }
        let out_line: String = output_fields.join(&self.output_delimiter);

        if self.wrap_lines.is_empty() {
            println!("{}\\\\", out_line);
        } else {
            println!("\\{}{{{}}}", self.wrap_lines, out_line);
        }
        Ok(())
    }

    fn cut<T: BufRead>(&self, reader: T) -> std::io::Result<()> {
        for line in reader.lines() {
            self.cut_line(&line?)?;
        }

        Ok(())
    }
}

fn escape_latex (value: &str) -> String {
    let mut builder = Builder::default();
    let specials = "~^\\";
    let simple_specials = "&%$#_{}";

    for ch in value.chars() {
        if specials.contains(ch) {
            builder.append(
                match ch {
                    '^'    => String::from("\\textasciitilde{}"),
                    '~'    => String::from("\\textasciicircum{}"),
                    '\\'   => String::from("\\textbackslash{}"),
                    _      => panic!("invalid condition"),
                }
            );
        } else {
            if simple_specials.contains(ch) {
                builder.append('\\');
            }
            builder.append(ch);
        }
    }
    builder.string().unwrap()
}


fn escape_listing (value: &str) -> String {
    let mut builder = Builder::default();
    let simple_specials = "\\_{}";

    for ch in value.chars() {
        if simple_specials.contains(ch) {
            builder.append('\\');
        }
        builder.append(ch);
    }
    builder.string().unwrap()
}

fn main() -> Result<(), String> {
    let mut options = CutOptions::new();
    let mut fields = String::new();
    let mut wrap_fields = String::new();
    let mut escape_fields = String::new();
    let mut input = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("formats a table-like stream into a LaTeX-table");
        ap.refer(&mut options.input_delimiter).add_option(&["-d", "--delimiter"], Store, "use this character for field delimiter").required();
        ap.refer(&mut fields).add_option(&["-f"], Store, "select only these fields;  also print any line that contains no delimiter character, unless the -s option is specified").required();
        ap.refer(&mut options.notonly_delimited).add_option(&["-s", "--only-delimited"], StoreTrue, "do not print lines not containing delimiters");
        //ap.refer(&mut options.complement).add_option(&["--complement"], StoreTrue, "complement the set of selected bytes, characters or fields");
        ap.refer(&mut options.wrap_lines).add_option(&["-l", "--wrap-lines"], Store, "use this LaTeX command to wrap lines");
        ap.refer(&mut wrap_fields).add_option(&["-w", "--wrap-fields"], Store, "comma separated list of LaTeX command names used to wrap fields");
        ap.refer(&mut escape_fields).add_option(&["-e", "--escape-fields"], Store, "comma separated list of encoding options, currently 'latex' and 'listing' are supported");
        ap.refer(&mut input).add_argument("FILE", Store, "name of the input file, or '-' to read from STDIN");
        ap.parse_args_or_exit();
    }
    options.fields = fields.split(",").map(|s| {s.parse::<usize>().expect("unable to read field ids")}).collect();
    options.wrap_fields = wrap_fields.split(",").map(|s| String::from(s)).collect();
    options.escape_fields = escape_fields.split(",").map(|s| match &s.to_lowercase()[..] {
        "latex"     => EncodingOption::LaTeX,
        "listing"   => EncodingOption::Listing,
        _           => EncodingOption::None
    }).collect();

    let res = 
    if input.is_empty() || input == "-" {
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
