# tmpl

A lightweight template programming language for creating dynamic templates and automating file generation.

## Table of Contents

- [Getting Started](#getting-started)
- [Syntax Overview](#syntax-overview)
- [Logical Statements](#logical-statements)
  - [Functions](#functions)
  - [Conditionals](#conditionals)
  - [Variables](#variables)
- [Filesystem Operations](#filesystem-operations)
- [Commands](#commands)
- [Built-in Functions](#built-in-functions)
- [Examples](#examples)

---

## Getting Started

`tmpl` is designed to simplify template creation and file generation through an intuitive syntax. Whether you're scaffolding projects, generating configuration files, or automating repetitive tasks, `tmpl` provides the tools you need.

> [!TIP]
> Start with simple templates and gradually add logic as your needs grow. The language is designed to be readable and maintainable.

---

## Syntax Overview

`tmpl` uses a straightforward syntax with clear keywords and minimal punctuation. All statements are designed to be self-documenting.

**Key principles:**

- Keywords are followed by colons (`:`)
- Code blocks use curly braces `{}`
- Function calls use parentheses `()`
- Comments start with `#` (implied from context)

---

## Logical Statements

### Functions

Define reusable blocks of logic with parameters.

**Syntax:**

```tmpl
function: [name]([arg1], [arg2]) {
    [logic]
}
```

**Example:**

```tmpl
function: create_component(name, type) {
    mkdir: components
    cd: components
    create_file: $name.$type
    write_file($name.$type): <<EOF
    export default function name() {
        return <div>Hello from name</div>
    }
    EOF>>
}
```

> [!NOTE]
> Functions can accept multiple parameters and contain any combination of tmpl statements.

---

### Conditionals

Control flow based on conditions. Supports both single-line and multi-line blocks.

**Single-line syntax:**

```tmpl
if: [statement] == [value] 
    [logic]
```

**Multi-line syntax:**

```tmpl
if: [statement] == [value] {
    [logic]
    [logic]
}
```

**Example:**

```tmpl
var: environment = input("Select environment", "development")

if: $environment == "production" {
    write_file(config.json): <<EOF
    {
        "debug": false,
        "apiUrl": "https://api.example.com"
    }
    EOF>>
}

if: $environment == "development"
    write_file(config.json): {"debug": true, "apiUrl": "http://localhost:3000"}
```

> [!IMPORTANT]
> Currently, only equality (`==`) comparison is supported. Ensure your conditions use exact value matching.

**Supported operators:**

- `==` - Equality check

---

### Variables

Store and reuse values throughout your template.

Variables can be called via $.

**Syntax:**

```tmpl
var: [name] = [value]
```

```tmpl
$[name]
```

**Example:**

```tmpl
var: project_name = input("Project name", "my-app")
var: author = "John Doe"
var: version = "1.0.0"

write_file(package.json): <<EOF
{
    "name": "$project_name",
    "author": "$author",
    "version": "$version"
}
EOF>>
```

> [!TIP]
> Use descriptive variable names with underscores for readability (e.g., `project_name` instead of `pn`).

---

## Filesystem Operations

Manipulate files and directories with simple commands.

### Create Directory

**Syntax:**

```tmpl
mkdir: [name]
```

**Example:**

```tmpl
mkdir: src
mkdir: public
mkdir: tests
```

---

### Create File

**Syntax:**

```tmpl
create_file: [name]
```

**Example:**

```tmpl
create_file: index.html
create_file: styles.css
create_file: app.js
```

> [!WARNING]
> Creating a file that already exists may overwrite the existing file. Always verify your template logic before execution.

---

### Write to File

Write content to a file. Use `<<EOF EOF>>` to write multiline.

**Syntax:**

```tmpl
write_file([filename]): <<EOF
[content]  
[more content]
EOF>>
```

**Example:**

```tmpl
create_file: README.md
write_file(README.md): <<EOF
# My Project 

Welcome to my project! 

## Installation

Run `npm install` to get started.
EOF>>
```

**Multi-line example:**

> [!NOTE]
> You can also use `\n` as a newline.

```tmpl
write_file(index.html): <!DOCTYPE html> \\n <html> \\n <head> \\n <title>My App</title> \\n </head> \\n <body> \\n <h1>Hello World</h1> \\n </body> \\n </html>
```

---

### Change Directory

Navigate between directories during template execution.

**Syntax:**

```tmpl
cd: [directory]
```

**Example:**

```tmpl
mkdir: src
cd: src
create_file: index.js
cd: ..
mkdir: tests
cd: tests
create_file: index.test.js
```

> [!CAUTION]
> Relative paths like `..` (parent directory) are supported, but ensure your path logic doesn't navigate outside the project root.

---

## Commands

Execute multiple commands in sequence using command blocks.

**Syntax:**

```tmpl
command
- [command] [...args]
- [command] [...args]
end_command
```

**Example:**

```tmpl
command
- npm init -y
- npm install express
- npm install --save-dev nodemon
end_command
```

**Another example:**

```tmpl
command
- git init
- git add .
- git commit -m "Initial commit"
end_command
```

> [!IMPORTANT]
> Commands are executed in the order they're defined. Each command must complete successfully before the next one runs.

---

## Built-in Functions

Pre-defined functions for common operations.

### input()

Prompt the user for input with an optional placeholder.

**Syntax:**

```tmpl
input([Question], [Placeholder])
```

**Parameters:**

- `Question` - The prompt text shown to the user
- `Placeholder` - Default value or hint text

**Example:**

```tmpl
var: app_name = input("What is your app name?", "my-awesome-app")
var: port = input("Which port should the server run on?", "3000")
var: use_typescript = input("Use TypeScript? (yes/no)", "no")

if: $use_typescript == "yes" {
    create_file: tsconfig.json
    write_file(tsconfig.json): { \\n "compilerOptions": { \\n "target": "ES2020" \\n } \\n }
}
```

> [!TIP]
> Use clear, specific questions in your input prompts. Good placeholders help users understand the expected format.

---

## Examples

### Example 1: Basic Project Setup

```tmpl
var: project = input("Project name", "my-project")

mkdir: $project
cd: $project

mkdir: src
mkdir: public
mkdir: tests

cd: src
create_file: index.js
write_file(index.js): console.log('Hello, World!'); \\n

cd: ..
create_file: README.md
write_file(README.md): # project \\n \\n A new project created with tmpl.
```

---

### Example 2: Express API Template

```tmpl
var: api_name = input("API name", "my-api")
var: port = input("Port number", "3000")

mkdir: $api_name
cd: $api_name

create_file: server.js
write_file(server.js): <<EOF
const express = require('express'); 
const app = express();

app.get('/', (req, res) => { 
    res.json({ message: 'Hello World' });
});

app.listen($port, () => { 
    console.log(`Server running on port $port`);
});
EOF>>

create_file: package.json
write_file(package.json): <<EOF
{ 
    "name": "api_name",
    "version": "1.0.0", 
    "main": "server.js", 
    "scripts": { 
        "start": "node server.js"
    }
}
EOF>>

command
- npm install express
end_command
```

---

### Example 3: Conditional Configuration

```tmpl
var: env = input("Environment (dev/prod)", "dev")
var: app_name = input("Application name", "myapp")

if: $env == "prod" {
    create_file: config.prod.json
    write_file(config.prod.json): <<EOF
    {
        "environment": "production",
        "debug": false,
        "apiUrl": "https://api.example.com"
    }
    EOF>>
}

if: $env == "dev" {
    create_file: config.dev.json
    write_file(config.dev.json): <<EOF 
    {  
        "environment": "development",
        "debug": true,
        "apiUrl": "http://localhost:3000"
    }
    EOF>>
}

create_file: .env
write_file(.env): APP_NAME=$app_name \\n ENVIRONMENT=$env
```

---

### Example 4: Component Generator with Function

```tmpl
function: create_react_component(name) {
    mkdir: components
    cd: components
    create_file: $name.jsx
    write_file($name.jsx): <<EOF
    import React from 'react';
    
    export default function name() {
        return ( 
            <div className="name">
                <h1>name Component</h1> 
            </div> 
        );    
    }
    EOF>>
    cd: ..
}

var: component_name = input("Component name", "Button")
create_react_component($component_name)
```

---

## Best Practices

> [!TIP]
> **Keep templates modular:** Break complex templates into functions for reusability.

> [!TIP]
> **Use meaningful variable names:** `project_name` is better than `pn`.

> [!TIP]
> **Validate user input:** Use conditionals to handle different input scenarios.

> [!WARNING]
> Always test your templates in a safe environment before using them on production projects.

---

## Contributing

Contributions are welcome! If you'd like to add features or improve documentation, please submit a pull request.

---

## License

[LICENCE](./LICENSE)
