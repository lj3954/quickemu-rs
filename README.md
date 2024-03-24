## NOTE: This project is not going to get much work on it for a while. I'm currently spending my time far more on my re-implementation of quickget, which will include many features that this project will benefit from.

## Purpose

This branch goes alongside my [quickget re-implementation](https://github.com/lj3954/qg-rust). 

The goal is to fully rewrite the [quickemu](https://github.com/quickemu-project/quickemu) bash script in Rust. 
It should be fully backwards compatible with the original script, but offer many improvements.

## Benefits

1. **Error messages**: The original quickemu bash script offers a very poor description of errors.
You can easily see this by visiting the issues tab of the project's repository, where nearly everyone
is shown multiple error messages, many of which are unhelpful. This project aims to fix that by providing
clear error messages with instructions on likely solutions, not unlike the Rust compiler.
2. **Reliability**: The original project, being a bash script, is not particularly reliable. 
There are many parts of the code which manipulate the strings in strange ways, which may cause issues.
This has been pointed out by many, and is an inherent problem in complex bash scripts.
3. **Features**: The original project's codebase is not not organized well enough to easily add new features.
I plan to add major features such as support for multiple architectures early on. Additionally, I hope to work
with the creator of [quickpassthrough](https://github.com/HikariKnight/quickpassthrough) to add support for GPU passthrough.

