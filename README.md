# tmpl

A Template programming language for creating templates

- [Definition of logical statements](#definition-of-logical-statements)
- [Filesystem commands](#filesystem-commands)
- [Commands](#commands)
- [Prebuilt commands](#prebuilt-commands)

## Definition of logical statements

Definition of a function

```tmpl
function: [name]([arg1], [arg2]) {
    [logic]
}
```

Definition of a conditional

```tmpl
if: [statement] == [value] 
    [logic]
```

***or***

```tmpl
if: [statement] == [value] {
    [logic]
    [logic]
}
```

Definition of a variable

```tmpl
var: [name] = [value]
```

## Filesystem commands

Creation of a new directory

```tmpl
mkdir: [name]
```

Creation of a new file

```tmpl
create_file: [name]
```

Writing to a file

```tmpl
write_file([filename]): [input] /n [input]
```

changing directories

```tmpl
cd: [directory]
```

## Commands

```tmpl
command
- [command] [...args]
- ...
end_command
```

## Prebuilt commands

```tmpl
input([Question], [Placeholder])
```
