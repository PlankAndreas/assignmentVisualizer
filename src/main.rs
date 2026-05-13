struct QDimacs {
    num_vars: usize,
    num_clauses: usize,
    quantifiers: Vec<QuantifierBlock>,
    clauses: Vec<Clause>,
}

enum QuantifierType {
    Exists,
    ForAll,
}

struct QuantifierBlock {
    qtype: QuantifierType,
    vars: Vec<i32>,
}

type Clause = Vec<i32>;

use std::fs::File;
use std::io::{BufRead, BufReader};

fn parse_prefix(line: &str) -> Option<(usize, usize)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() == 4 && parts[0] == "p" && parts[1] == "cnf" {
        let num_vars = parts[2].parse::<usize>().ok()?;
        let num_clauses = parts[3].parse::<usize>().ok()?;
        Some((num_vars, num_clauses))
    } else {
        None
    }
}

fn parse_quantifier(line: &str) -> QuantifierBlock {
    let qtype = if line.starts_with('a') {
        QuantifierType::ForAll
    } else {
        QuantifierType::Exists
    };
    let vars: Vec<i32> = line
        .split_whitespace()
        .skip(1)    
        .filter_map(|s| s.parse::<i32>().ok())
        .take_while(|&x| x != 0)
        .collect();
    QuantifierBlock { qtype, vars }
}

fn parse_clause(line: &str) -> Clause {
    line.split_whitespace()
        .filter_map(|s| s.parse::<i32>().ok())
        .take_while(|&x| x != 0)
        .collect()
}

fn parse_file(path: &str) -> std::io::Result<QDimacs> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut header: Option<(usize, usize)> = None;
    let mut quantifiers = Vec::new();
    let mut clauses = Vec::new();

    for line in reader.lines() {
        let line = line?;

        if line.starts_with('c') || line.trim().is_empty() {
            continue;
        }

        // STEP 1: prefix
        if header.is_none() {
            if let Some(h) = parse_prefix(&line) {
                header = Some(h);
                continue;
            }
        }

        // 2. quantifiers
        if line.starts_with('a') || line.starts_with('e') {
            let qblock = parse_quantifier(&line);
            quantifiers.push(qblock);
            continue;
        }

        // 3. clauses
        else if line.chars().next().unwrap().is_digit(10) || line.starts_with('-') {
        let clause = parse_clause(&line);
        clauses.push(clause);
}

        println!("{line}");
    }
    
    let (num_vars, num_clauses) = header.unwrap();
    let qdimacs:QDimacs = QDimacs {
        num_vars,
        num_clauses,
        quantifiers,
        clauses,
    };

    Ok(qdimacs)
}

fn print_qdimacs(qdimacs: &QDimacs) {
    println!("p cnf {} {}", qdimacs.num_vars, qdimacs.num_clauses);
    for q in &qdimacs.quantifiers {
        let qtype = match q.qtype {
            QuantifierType::Exists => "e",
            QuantifierType::ForAll => "a",
        };
        let vars: Vec<String> = q.vars.iter().map(|v| v.to_string()).collect();
        println!("{} {} 0", qtype, vars.join(" "));
    }
    for clause in &qdimacs.clauses {
        let literals: Vec<String> = clause.iter().map(|l| l.to_string()).collect();
        println!("{} 0", literals.join(" "));
    }
}

fn main() {
    let qdimacs = parse_file("example.qdimacs").unwrap();
    print_qdimacs(&qdimacs);
    println!("Hello, world!");
}
