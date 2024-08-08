import hlbc

# Open and disassemble a file
code = hlbc.Bytecode("./Clazz.hl")

# Get the debug files
print(code.get_debug_files())
assert code.get_debug_files() == ['Clazz.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/_std/Std.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/_std/String.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/_std/StringBuf.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/_std/Type.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/_std/haxe/Exception.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/_std/haxe/NativeStackTrace.hx', 'C:\\HaxeToolkit\\haxe\\std/haxe/ds/ArraySort.hx', 'C:\\HaxeToolkit\\haxe\\std/haxe/exceptions/PosException.hx', 'C:\\HaxeToolkit\\haxe\\std/haxe/exceptions/NotImplementedException.hx', 'C:\\HaxeToolkit\\haxe\\std/haxe/iterators/ArrayIterator.hx', 'C:\\HaxeToolkit\\haxe\\std/haxe/iterators/ArrayKeyValueIterator.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/BaseType.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/Bytes.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/NativeArray.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/types/ArrayBase.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/types/ArrayBytes.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/types/ArrayDyn.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/types/ArrayObj.hx', '?']
# > ['Clazz.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/_std/Std.hx', 'C:\\HaxeToolkit\\haxe\\std/hl/_std/String.hx', ... '?']

# Get all functions from a specific debug file in the bytecode
print(code.get_functions("Clazz.hx"))
assert code.get_functions("Clazz.hx") == ['fn main@22 () -> void', 'fn method@23 (Clazz) -> i32', 'fn <none>@337 ((f64, f64) -> i32, i32, i32) -> i32', 'fn <none>@341 ((dynamic, dynamic) -> i32, f64, f64) -> i32', 'fn <none>@343 ((i32, i32) -> i32, f64, f64) -> i32', 'fn <none>@347 ((dynamic, dynamic) -> i32, i32, i32) -> i32', 'fn <none>@349 ((f32, f32) -> i32, i32, i32) -> i32', 'fn <none>@350 ((f32, f32) -> i32, f64, f64) -> i32', 'fn <none>@354 ((dynamic, dynamic) -> i32, f32, f32) -> i32', 'fn <none>@356 ((i16, i16) -> i32, i32, i32) -> i32', 'fn <none>@357 ((i16, i16) -> i32, f64, f64) -> i32', 'fn <none>@361 ((dynamic, dynamic) -> i32, i16, i16) -> i32']
# > ['fn main@22 () -> void', 'fn method@23 (Clazz) -> i32', 'fn <none>@337 ((f64, f64) -> i32, i32, i32) -> i32', ...]

# Decompile a single function by index
print(code.decompile("23"))
assert code.decompile("23").strip() == """static function method(_: Clazz): Int {
  return 42;
}"""
# > static function method(_: Clazz): Int {
# >  return 42;
# > }

# Stub a function
print(code.stub("23"))
assert code.stub("23").strip() == "static function method(_: Clazz): Int {}"
# > static function method(_: Clazz): Int {}

print(code.class_named("Clazz"))
assert code.class_named("Clazz") == "128"
print(code.stub_class(code.class_named("Clazz")))