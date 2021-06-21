use crate::drawing::FmtChar;
use crate::drawing::FmtString;

fn read_until(ch: char, data: &Vec<char>, mut pos: usize) -> (Vec<char>, usize) {
    if pos >= data.len() - 1 {
        return (vec![], pos);
    }
    let start = pos;
    while pos < data.len() && data[pos] != ch {
    	pos += 1;
    }
    if pos >= data.len() - 1 {
        return (data[start..pos].to_vec(), pos);
    } 
    (data[start..pos].to_vec(), pos)
}

pub fn make_data(data: Vec<char>) -> FmtString {
    let mut out: Vec<FmtChar> = Vec::new();
    let mut pos = 0;
    let mut bg = "".to_string();
    let mut fg = "".to_string();

    while pos < data.len() {
        if data[pos] == '\u{001b}' {
            pos += 2;
            if data[pos..pos + 2].to_vec().into_iter().collect::<String>() == "0m" {
                bg = "".to_string();
                fg = "".to_string();
                pos += 2;
                continue;
            }
            if data[pos..pos + 3].to_vec().into_iter().collect::<String>() == "39m" {
                bg = "".to_string();
                fg = "".to_string();
                pos += 3;
                continue;
            }
            if data[pos..pos + 3].to_vec().into_iter().collect::<String>() == "49m" {
                bg = "".to_string();
                fg = "".to_string();
                pos += 3;
                continue;
            }
            let (first_num,   npos) = read_until(';', &data, pos); pos = npos;
            let (_second_num, npos) = read_until(';', &data, pos + 1); pos = npos;

            let (rv, npos) = read_until(';', &data, pos + 1); pos = npos;
            let (gv, npos) = read_until(';', &data, pos + 1); pos = npos;
            let (bv, npos) = read_until('m', &data, pos + 1); pos = npos;
            let r = rv.into_iter().collect::<String>().parse::<u8>().unwrap();
            let g = gv.into_iter().collect::<String>().parse::<u8>().unwrap();
            let b = bv.into_iter().collect::<String>().parse::<u8>().unwrap();
            pos += 1;
            let s: String = first_num.into_iter().collect();
            if s == "38" {
                fg = termion::color::Rgb(r, g, b).fg_string();
            } else if s == "48" {
                bg = termion::color::Rgb(r, g, b).fg_string()
            }
        } else {
            out.push(FmtChar{ch: data[pos].to_string(), fg: fg.clone(), bg: bg.clone()});
            pos += 1;
        }
    }
    FmtString::from_buffer(out)
}