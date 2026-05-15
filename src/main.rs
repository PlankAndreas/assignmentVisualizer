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

use std::collections::HashMap;

type Assignment = HashMap<i32, bool>;

fn count_models(q: &QDimacs) -> u64 {
    let vars: Vec<i32> = q
    .quantifiers
    .iter()
    .flat_map(|block| block.vars.iter().copied())
    .collect();

    fn eval_clause(clause: &Clause, assignment: &Assignment) -> bool {
        for &lit in clause {
            let var = lit.abs();
            let sign = lit > 0;

            if let Some(&val) = assignment.get(&var) {
                if val == sign {
                    return true; // clause satisfied
                }
            }
        }
        false
    }

    fn eval_formula(q: &QDimacs, assignment: &Assignment) -> bool {
        q.clauses.iter().all(|clause| eval_clause(clause, assignment))
    }

    fn dfs(
        q: &QDimacs,
        vars: &[i32],
        depth: usize,
        assignment: &mut Assignment,
        eval_formula: &dyn Fn(&QDimacs, &Assignment) -> bool,
        dot: &mut String,
        next_id: &mut usize,
    ) -> (u64, usize) {
        let my_id = *next_id;
        *next_id += 1;
        // leaf: all variables assigned
        if depth == vars.len() {
            let value = if eval_formula(q, assignment) { 1 } else { 0 };

            let color = if value == 1 { "green" } else { "gray" };

            dot.push_str(&format!(
                r#"{id} [label="{value}", shape=box, color="{color}"];"#,
                id = my_id
            ));

            return (value, my_id);
        }

        let var = vars[depth];

        // left branch: false
        assignment.insert(var, false);
        let (left, left_id) = dfs(
            q,
            vars,
            depth + 1,
            assignment,
            eval_formula,
            dot,
            next_id,
        );
        // right branch: true
        assignment.insert(var, true);
        let (right, right_id) = dfs(
            q,
            vars,
            depth + 1,
            assignment,
            eval_formula,
            dot,
            next_id,
        );
        let is_universal = q.quantifiers.iter().any(|block| {
            matches!(block.qtype, QuantifierType::ForAll)
            && block.vars.contains(&var)
        });
        let weight = if is_universal {
            left * right
        } else {
            left + right
        };
        let qsymbol = if is_universal { "A" } else { "E" };
        let color = if is_universal { "red" } else { "blue" };

        dot.push_str(&format!(
            r#"{id} [label="x{var}\n{q}\nw={w}", color="{color}"];"#,
            id = my_id,
            var = var,
            q = qsymbol,
            w = weight,
        ));

        dot.push_str(&format!(
            r#"{parent} -> {child} [label="0"];"#,
            parent = my_id,
            child = left_id,
        ));

        dot.push_str(&format!(
            r#"{parent} -> {child} [label="1"];"#,
            parent = my_id,
            child = right_id,
        ));
        (weight, my_id)
    }

    let mut assignment = HashMap::new();
    let mut dot = String::from("digraph G {\n");
    dot.push_str("node [shape=circle];\n");

    let mut next_id = 0;

    let (result, _) = dfs(
        q,
        &vars,
        0,
        &mut assignment,
        &eval_formula,
        &mut dot,
        &mut next_id,
    );
    dot.push_str("}\n");

    fn open_image(path: &str) {
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/C", "start", "", path])
                .status()
                .expect("failed to open image");
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(path)
                .status()
                .expect("failed to open image");
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(path)
                .status()
                .expect("failed to open image");
        }
    }

    std::fs::write("tree.dot", dot).unwrap();
    use std::process::Command;
    Command::new("dot")
        .args(&["-Tsvg", "tree.dot", "-o", "tree.svg"])
        .output()
        .expect("Failed to execute dot command");
    Command::new("dot")
        .args(["-Tpng", "tree.dot", "-o", "tree.png"])
        .status()
        .expect("Failed to run dot");

    open_image("tree.svg");
    result
}

fn main() {
    let qdimacs = parse_file("example.qdimacs").unwrap();
    print_qdimacs(&qdimacs);

    let count = count_models(&qdimacs);

    println!("Model count: {}", count);

    println!("Hello, world!");
}
