use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::collections::HashMap;
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::process::Stdio;
use std::env;
use crate::iostream;
use std::fs;
use dirs;
use std::thread;

pub struct FunctionDefinition {
    pub start_line: usize,  
    pub end_line: usize,    
    pub params: Vec<String>, 
}

fn parse_file(name: &str) -> io::Result<String> {
    let mut path: PathBuf = dirs::data_dir()
        .expect("Could not find data directory")
        .join("tmpl/templates");
    path.push(format!("{}/file.tmpl", name));

    let mut file = File::open(&path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?; 

    Ok(contents)
}

pub fn parse_template(template: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_path = std::env::current_dir()?;
    let mut variables: HashMap<String, String> = HashMap::new();
    let mut functions: HashMap<String, FunctionDefinition> = HashMap::new();
    let file_contents = parse_file(template)?;
    let lines: Vec<_> = file_contents.lines().collect();   

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        
        if line.starts_with("function:") {
            let (func_name, func_def, end_index) = parse_function_definition(&lines, i);
            functions.insert(func_name, func_def);
            i = end_index + 1;
        } else {
            i += 1;
        }
    }

    let mut i = 0;
    let mut is_command = false;

    while i < lines.len() {
        let line = lines[i].trim();
        
        if line.starts_with("function:") {
            let mut brace_line = i;
            if !line.ends_with('{') {
                brace_line += 1;
                while brace_line < lines.len() && lines[brace_line].trim() != "{" {
                    brace_line += 1;
                }
            }
            
            let mut depth = 1;
            i = brace_line + 1;
            while i < lines.len() && depth > 0 {
                let trimmed = lines[i].trim();
                if trimmed == "{" {
                    depth += 1;
                } else if trimmed == "}" {
                    depth -= 1;
                }
                i += 1;
            }
            continue;
        }

        let (new_is_command, skip_lines) = parse_line_and_execute(
            lines[i], 
            is_command, 
            &mut variables,
            &functions,
            &lines,
            i
        );
        is_command = new_is_command;
        i += skip_lines + 1;
    }

    return Ok(())
}

fn parse_line_and_execute(
    line: &str, 
    is_command: bool, 
    variables: &mut HashMap<String, String>,
    functions: &HashMap<String, FunctionDefinition>,
    all_lines: &[&str],
    current_index: usize
) -> (bool, usize) {
    let line = line.trim();

    if line.starts_with("var:") {
        handle_var(line, variables);
    } else if line.starts_with("mkdir:") {
        handle_mkdir(line, variables);
    } else if line.starts_with("create_file:") {
        handle_create_file(line, variables);
    } else if line.starts_with("write_file(") {
        handle_write_file(line, variables, all_lines, current_index);
    } else if line.starts_with("command") {
        return (true, 0);
    } else if line.starts_with("end_command") {
        return (false, 0);
    } else if line.starts_with("-") {
        handle_command_line(line, is_command, variables);
        return (true, 0);
    } else if line.starts_with("cd:") {
        handle_cd(line, variables);
    } else if line.starts_with("if:") {
        let skip = handle_if(line, is_command, variables, all_lines, current_index);
        return (is_command, skip);
    } else if line.starts_with("#") {
        // Comment line, ignore
    } else if line == "{" || line == "}" {
        // Block delimiters, ignore
    } else if line.is_empty() {
        // Skip empty lines
    } else if is_function_call(line, functions) {
        // Check if it's a function call
        handle_function_call(line, variables, functions, all_lines);
    } else {
        println!("Unknown command: {}", line);
    }
    
    (false, 0)
}

fn replace_variables(text: &str, variables: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    
    for (key, value) in variables {
        let placeholder = format!("${}", key);
        result = result.replace(&placeholder, value);

        result = result.replace(&format!("$${}", key), &format!("${}", key));
    }
    
    result
}

fn parse_function_definition(
    lines: &[&str], 
    start_index: usize
) -> (String, FunctionDefinition, usize) {
    let line = lines[start_index].trim();
    
    let func_decl = line[9..].trim(); 
    
    let has_brace_on_same_line = func_decl.ends_with('{');
    
    let func_decl = func_decl.trim_end_matches('{').trim();
    
    let (name, params) = if func_decl.contains('(') {
        let paren_pos = func_decl.find('(').unwrap();
        let name = func_decl[..paren_pos].trim().to_string();
        
        let params_end = func_decl.rfind(')').unwrap_or(func_decl.len());
        let params_str = &func_decl[paren_pos + 1..params_end].trim();
        
        let params: Vec<String> = if params_str.is_empty() {
            vec![]
        } else {
            params_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };
        
        (name, params)
    } else {
        (func_decl.to_string(), vec![])
    };

    let brace_line = if has_brace_on_same_line {
        start_index
    } else {
        let mut line_idx = start_index + 1;
        while line_idx < lines.len() {
            let trimmed = lines[line_idx].trim();
            if trimmed == "{" {
                break;
            } else if !trimmed.is_empty() && !trimmed.starts_with("//") {
                eprintln!("Expected '{{' after function declaration for '{}'", name);
                return (name, FunctionDefinition {
                    start_line: start_index + 1,
                    end_line: start_index + 1,
                    params,
                }, start_index);
            }
            line_idx += 1;
        }
        
        if line_idx >= lines.len() {
            eprintln!("Missing opening '{{' for function '{}'", name);
            return (name, FunctionDefinition {
                start_line: start_index + 1,
                end_line: start_index + 1,
                params,
            }, start_index);
        }
        
        line_idx
    };

    let mut depth = 1;
    let mut end_index = brace_line + 1;
    
    while end_index < lines.len() && depth > 0 {
        let trimmed = lines[end_index].trim();
        
        if trimmed.is_empty() || trimmed.starts_with("//") {
            end_index += 1;
            continue;
        }
        
        if trimmed == "{" {
            depth += 1;
        } else if trimmed == "}" {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
        
        end_index += 1;
    }

    if depth != 0 {
        eprintln!("Missing closing '}}' for function '{}'", name);
    }

    let func_def = FunctionDefinition {
        start_line: brace_line + 1,  
        end_line: end_index,         
        params,
    };

    (name, func_def, end_index)
}

fn is_function_call(line: &str, functions: &HashMap<String, FunctionDefinition>) -> bool {
    if !line.contains('(') || !line.ends_with(')') {
        return false;
    }
    
    let name_part = line.split('(').next().unwrap();
    
    if name_part.is_empty() || name_part.contains(':') {
        return false;
    }
    
    let is_valid_identifier = name_part.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_')
        && name_part.chars().all(|c| c.is_alphanumeric() || c == '_');
    
    is_valid_identifier && functions.contains_key(name_part)
}

fn handle_function_call(
    line: &str,
    variables: &mut HashMap<String, String>,
    functions: &HashMap<String, FunctionDefinition>,
    all_lines: &[&str]
) {
    let parts: Vec<&str> = line.split('(').collect();
    let func_name = parts[0].trim();
    let args_str = parts[1].trim_end_matches(')').trim();

    let args: Vec<String> = if args_str.is_empty() {
        vec![]
    } else {
        args_str
            .split(',')
            .map(|s| {
                let trimmed = s.trim();
                replace_variables(trimmed, variables)
            })
            .collect()
    };

    let func_def = match functions.get(func_name) {
        Some(def) => def,
        None => {
            eprintln!("Function '{}' not found", func_name);
            return;
        }
    };

    if args.len() != func_def.params.len() {
        eprintln!(
            "Function '{}' expects {} parameter(s), but {} were provided",
            func_name,
            func_def.params.len(),
            args.len()
        );
        return;
    }

    let mut local_vars = variables.clone();
    
    for (param_name, arg_value) in func_def.params.iter().zip(args.iter()) {
        local_vars.insert(param_name.clone(), arg_value.clone());
    }

    let mut i = func_def.start_line;
    let mut is_command = false;
    
    while i < func_def.end_line {
        let (new_is_command, skip_lines) = parse_line_and_execute(
            all_lines[i],
            is_command,
            &mut local_vars,
            functions, 
            all_lines,
            i
        );
        is_command = new_is_command;
        i += skip_lines + 1;
    }
}

fn handle_function(
    line: &str, 
    is_command: bool, 
    variables: &HashMap<String, String>,
    functions: &HashMap<String, FunctionDefinition>,
    all_lines: &[&str],
    current_index: usize
) -> usize {
    let func_name = &line[9..].trim();

    if *func_name == "example_function" {
        println!("Executing example_function");
    } else {
        eprintln!("Unknown function: {}", func_name);
    }

    0
}

fn handle_if(
    line: &str, 
    is_command: bool, 
    variables: &HashMap<String, String>,
    all_lines: &[&str],
    current_index: usize
) -> usize {
    let condition = &line[3..].trim();
    let condition = replace_variables(condition, variables);

    let parts: Vec<&str> = condition.splitn(2, "==").collect();
    if parts.len() != 2 {
        eprintln!("Invalid if condition: {}", line);
        return 0;
    }

    let left = parts[0].trim();
    let right = parts[1].trim();

    let condition_true = left == right;

    if !condition_true {
        if current_index + 1 < all_lines.len() {
            let next_line = all_lines[current_index + 1].trim();
            
            if next_line == "{" {
                let mut depth = 1;
                let mut skip_count = 1; 
                
                for i in (current_index + 2)..all_lines.len() {
                    let block_line = all_lines[i].trim();
                    
                    if block_line == "{" {
                        depth += 1;
                    } else if block_line == "}" {
                        depth -= 1;
                        if depth == 0 {
                            skip_count = i - current_index;
                            break;
                        }
                    }
                }
                
                return skip_count;
            } else {
                return 1;
            }
        }
    }

    0
}

fn handle_var(line: &str, variables: &mut HashMap<String, String>) {
    let name_and_value = &line[4..];
    
    let parts: Vec<&str> = name_and_value.splitn(2, '=').collect();
    let var_name = parts[0].trim();
    let var_value = if parts.len() > 1 {
        let raw_value = parts[1].trim();
        if raw_value.starts_with("input(") {
            let question_and_default = raw_value[6..raw_value.len()-1].to_string();
            let parts: Vec<&str> = question_and_default.splitn(2, ',').collect();
            let question = parts[0].trim();
            let default_value = if parts.len() > 1 {
                parts[1].trim()
            } else {
                ""
            };

            let input = iostream::get_input_text(&question, default_value).unwrap();
            input
        } else {
            replace_variables(raw_value, variables)
        }
    } else {
        String::new()
    };
    
    variables.insert(var_name.to_string(), var_value);
}

fn handle_mkdir(line: &str, variables: &HashMap<String, String>) {
    let name = &line[6..];
    let name = replace_variables(name, variables);
    let name = name.trim();

    let running = Arc::new(AtomicBool::new(true));
    let loader_flag = running.clone();

    let loader = thread::spawn(move || {
        iostream::show_loader("Creating directory \x1b[90m...\x1b[0m", loader_flag);
    });

    if let Err(e) = fs::create_dir_all(&name) {
        eprintln!("Failed to create directory '{}': {}", name, e);
    }

    running.store(false, Ordering::Relaxed);
    loader.join().unwrap();
}

fn handle_create_file(line: &str, variables: &HashMap<String, String>) {
    let name = &line[12..];
    let name = replace_variables(name, variables);
    let name = name.trim();

    let running = Arc::new(AtomicBool::new(true));
    let loader_flag = running.clone();

    let loader = thread::spawn(move || {
        iostream::show_loader("Creating file \x1b[90m...\x1b[0m", loader_flag);
    });

    if let Err(e) = File::create(&name) {
        eprintln!("Failed to create file '{}': {}", name, e);
    }

    running.store(false, Ordering::Relaxed);
    loader.join().unwrap();
}

fn handle_write_file(
    line: &str,
    variables: &HashMap<String, String>,
    all_lines: &[&str],
    current_index: usize
) {
    let parts: Vec<&str> = line.splitn(2, "):").collect();
    if parts.len() != 2 {
        eprintln!("Invalid write_file syntax: {}", line);
        return;
    }

    let file_name_raw = parts[0]
        .trim_start_matches("write_file(")
        .trim();

    let file_name = replace_variables(file_name_raw, variables);
    let content = replace_variables(parts[1].trim(), variables);

    fn unescape(s: &str) -> String {
        s.replace("\\n", "\n")
         .replace("\\t", "\t")
         .replace("\\r", "\r")
    }

    let content = unescape(&content);

    let content = if content.starts_with("<<EOF") {
        let mut collected = String::new();
        let mut i = current_index + 1;
        while i < all_lines.len() {
            let line = all_lines[i];
            if line.trim() == "EOF>>" {
                break;
            }
            collected.push_str(line);
            collected.push('\n');
            i += 1;
        }
        collected
    } else {
        content
    };

    let running = Arc::new(AtomicBool::new(true));
    let loader_flag = running.clone();

    let loader = thread::spawn(move || {
        iostream::show_loader("Writing file \x1b[90m...\x1b[0m", loader_flag);
    });

    let result = (|| -> std::io::Result<()> {
        let mut file = File::create(&file_name)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    })();

    if let Err(e) = result {
        eprintln!("Failed to write to file '{}': {}", file_name, e);
    }

    running.store(false, Ordering::Relaxed);
    loader.join().unwrap();
}

fn handle_command_line(line: &str, is_command: bool, variables: &HashMap<String, String>) {
    if !is_command {
        println!("Error: Command line outside of command block: {}", line);
        return;
    }

    let command = line[2..].trim();
    let command = replace_variables(command, variables);
    let value = command.clone();

    if command.is_empty() {
        eprintln!("Empty command");
        return;
    }

    let args: Vec<&str> = command.split_whitespace().collect();

    let running = Arc::new(AtomicBool::new(true));
    let loader_flag = running.clone();

    let loader = thread::spawn(move || {
        iostream::show_loader(&format!("Running command {} \x1b[90m...\x1b[0m", value), loader_flag);
    });

    let result = {
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .current_dir(std::env::current_dir().unwrap())
                .args(&["/C", &command])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
        }

        #[cfg(not(target_os = "windows"))]
        {
            Command::new(args[0])
                .current_dir(std::env::current_dir().unwrap())
                .args(&args[1..])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
        }
    };

    match result {
        Ok(status) => {
            if !status.success() {
                eprintln!("Command '{}' failed with exit code {:?}", command, status.code());
            }
        }
        Err(e) => {
            eprintln!("Failed to execute command '{}': {}", command, e);
        }
    }

    running.store(false, Ordering::Relaxed);
    loader.join().unwrap();
}


fn handle_cd(line: &str, variables: &HashMap<String, String>) {
    let dir = &line[3..];
    let dir = replace_variables(dir, variables);
    let dir = dir.trim();
    let dir = dir.to_string();
    let value = dir.clone();    

    let running = Arc::new(AtomicBool::new(true));
    let loader_flag = running.clone();

    let loader = thread::spawn(move || {
        iostream::show_loader(&format!("Changing directory to '{}'", value), loader_flag);
    });

    if let Err(e) = env::set_current_dir(&dir) {
        eprintln!("Failed to change directory to '{}': {}", dir, e);
    }

    running.store(false, Ordering::Relaxed);
    loader.join().unwrap();
}
