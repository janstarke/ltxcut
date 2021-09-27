use argparse::{ArgumentParser, Store, StoreTrue};
use std::path::Path;
use std::io::{BufRead, BufReader};
use std::fs::File;
use string_builder::Builder;
use std::collections::{HashMap};
use std::cell::RefCell;

#[derive(Clone)]
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

struct Converter<T> where T: Fn(usize) -> Box<dyn Fn(String) -> String> {
    converters: RefCell<HashMap<usize, Box<dyn Fn(String) -> String>>>,
    get_converter: T,
}

impl<T> Converter <T> where T: Fn(usize) -> Box<dyn Fn(String) -> String> {
    fn new (get_converter: T) -> Converter<T> {
        Converter {
            converters: RefCell::new(HashMap::new()),
            get_converter,
        }
    }

    fn call(&self, idx: usize, value: String) -> String {
        let mut converter = self.converters.borrow_mut();
        match converter.get(&idx) {
            Some(v)     =>  (v)(value),
            None        =>  {
                let v = (self.get_converter)(idx);
                let result = (&v)(value);
                converter.insert(idx, v);
                result
            }
        }
    }
}

fn field_wrapper(fields: &Vec<String>, id: usize) -> Box<dyn Fn(String) -> String> {
    if id >= fields.len() {
        return Box::new(|s| s);
    }
    let wrap_function = fields[id].clone();
    
    if wrap_function.is_empty() {
        Box::new(|s| s)
    } else {
        Box::new(move |s| String::from(format!("\\{}{{{}}}", wrap_function, s)))
    }
}

fn field_encoder (fields: &Vec<EncodingOption>, id: usize) -> Box<dyn Fn(String) -> String> {
    if id >= fields.len() {
        return Box::new(|s| s);
    }
    match fields[id] {
        EncodingOption::LaTeX   => Box::new(escape_latex),
        EncodingOption::Listing => Box::new(escape_listing),
        EncodingOption::None    => Box::new(|s| s)
    }
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

    fn cut_line(&self, line: &str, converter: &Box<dyn Fn(usize, String) -> String>) -> std::io::Result<()> {
        if ! line.contains(&self.input_delimiter) {
            if ! self.notonly_delimited {
                println!("{}", line);
            }
            return Ok(())
        }

        let input_fields: Vec<&str> = line.split(&self.input_delimiter).collect();
        let mut output_fields: Vec<String> = Vec::new();

        for output_column in 0..self.fields.len() {
            let input_column = self.fields[output_column] - 1;

            output_fields.push((converter)(output_column,
                if input_column < input_fields.len() {
                    String::from(input_fields[input_column])
                } else {
                    String::new()
                }
            ));
        }
        let out_line: String = output_fields.join(&self.output_delimiter);

        if self.wrap_lines.is_empty() {
            println!("{}\\\\", out_line);
        } else {
            println!("\\{}{{{}}}", self.wrap_lines, out_line);
        }
        Ok(())
    }

    fn cut<'a, T: BufRead>(&'a self, reader: T) -> std::io::Result<()> {
        let f = self.wrap_fields.clone();
        let wrapper = Converter::new(move |i| field_wrapper(&f, i));

        let f = self.escape_fields.clone();
        let encoder = Converter::new(move |i| field_encoder(&f, i));

        let converter:Box<dyn Fn(usize, String) -> String> = Box::new(move |i,v| wrapper.call(i, encoder.call(i, v)));

        for line in reader.lines() {
            self.cut_line(&line?, &converter)?;
        }

        Ok(())
    }
}

fn escape_latex (value: String) -> String {
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


fn escape_listing (value: String) -> String {
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
        ap.set_description("Formats a table-like stream into a LaTeX-table. Input is considered to be UTF-8 encoded");
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
        "none"      => EncodingOption::None,
        ""          => EncodingOption::LaTeX,
        "listing"   => EncodingOption::Listing,
        "latex"     => EncodingOption::LaTeX,
        _           => panic!("unknown encoding specified: {}", &s),
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
