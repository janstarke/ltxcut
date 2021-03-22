# ltxcut
formats a table-like stream into a LaTeX-table

## Usage:

```
Usage:
  ltxcut [OPTIONS] [FILE]

formats a table-like stream into a LaTeX-table

Positional arguments:
  FILE                  name of the input file, or '-' to read from STDIN

Optional arguments:
  -h,--help             Show this help message and exit
  -d,--delimiter DELIMITER
                        use this character for field delimiter
  -f                    select only these fields; also print any line that
                        contains no delimiter character, unless the -s option
                        is specified
  -s,--only-delimited   do not print lines not containing delimiters
  -l,--wrap-lines WRAP_LINES
                        use this LaTeX command to wrap lines
  -w,--wrap-fields WRAP_FIELDS
                        comma separated list of LaTeX command names used to
                        wrap fields
  -e,--escape-fields ESCAPE_FIELDS
                        comma separated list of encoding options, currently
                        'latex' and 'listing' are supported
```

## Example

```bash
$ cat sample.csv
TEST
a;b;C:\Windows\test_123.txt
1;17,8%;3

$ ltxcut sample.csv -f 1,2,3 -d ';' -w ,ts,filename -l mmsline -e ,latex,listing sample.csv

TEST
\mmsline{a & \ts{b} & \filename{C:\\Windows\\test\_123.txt}}
\mmsline{1 & \ts{17,8\%} & \filename{3}}
\mmsline{ & \ts{hello world} & \filename{test}}
```