# hlbc-python

A (terrible) Python wrapper for [hlbc](https://github.com/Gui-Yom/hlbc) - A Hashlink bytecode disassembler and decompiler.

## Installation

```bash
pip install hlbc
```

## Usage

Before running this example, download either the [prebuilt bytecode](./test/Clazz.hl) or the [source code](./test/Clazz.hx) of the `Clazz` test file and place the compiled bytecode in the same directory as the script.

```python
import hlbc

# Open and disassemble a file
code = hlbc.Bytecode("./Clazz.hl")

# Get the debug files
print(code.get_debug_files())
# > ['Clazz.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/_std/Std.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/_std/String.hx', ... '?']

# Get all functions from a specific debug file in the bytecode
print(code.get_functions("Clazz.hx"))
# > ['fn main@22 () -> void', 'fn method@23 (Clazz) -> i32', 'fn <none>@337 ((f64, f64) -> i32, i32, i32) -> i32', ...]

# Decompile a single function by index
print(code.decompile("23"))
# > static function method(_: Clazz): Int {
# >  return 42;
# > }

# Stub a function
print(code.stub("23"))
# > static function method(_: Clazz): Int {}
```

## Notes

- Any errors will be raised back to Python as simple `Exception`s, but this is subject to change.
- The `Bytecode` class is thread-safe. You can use the same instance multiple times in different threads. It also doesn't hold the file open, so you can create multiple instances of it with the same file.

<!-- TODO: actual docs? pdoc3? who the hell knows... -->