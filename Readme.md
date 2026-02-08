# diran - A high-performance directory analyzer

## Quick Start

```cmd

dira size --top 100

dira loc --path ..\myproject --top 100

```

## Purpose

dira is a cli tool for studying basic cli tool in Rust.  
It Teaches:

- How to interact with file
- How to write a markdown file
- Git basics
- Rust project structure


### Loc command
```cmd

dira loc --ignore rs,py
dira loc --only rs --top 50

```
### Size command
This command will calculate accumulating size in parallel using **ramyon** crate. Normally it will use maximum threads your cpu provides, but if the target directory is an HDD the thread pool will be limited to two threads
```cmd

dira size --top 10
dira size
```
