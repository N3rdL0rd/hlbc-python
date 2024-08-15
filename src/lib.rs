use hlbc::Bytecode as _Bytecode;
use pyo3::prelude::*;
use hlbc::fmt::EnhancedFmt;
use hlbc_decompiler::ast::{Class, ClassField, Method};
use hlbc::types::{RefFun, Type, Function, RefField, TypeObj};
use std::sync::Arc;
use std::panic::{self, AssertUnwindSafe};

#[pyclass]
struct Bytecode {
    bytecode: Arc<hlbc::Bytecode>,
}

fn catch_panic<F: FnOnce() -> PyResult<T>, T>(f: F) -> PyResult<T> {
    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(panic_error) => {
            let _error_msg = panic_error.downcast_ref::<String>()
                .map(|s| s.as_str())
                .or_else(|| panic_error.downcast_ref::<&str>().map(|s| *s))
                .unwrap_or("Unknown panic occurred");
            Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!("Panic!")))
        }
    }
}
fn stub_function(_code: &_Bytecode, f: &Function) -> Method {
    Method {
        fun: f.findex,
        static_: true,
        dynamic: false,
        statements: Vec::new(),
    }
}

fn stub_class(code: &_Bytecode, obj: &TypeObj) -> (Class, Vec<usize>) {
    let static_type = obj.get_static_type(code);

    let mut fields = Vec::new();
    for (i, f) in obj.own_fields.iter().enumerate() {
        if obj
            .bindings
            .contains_key(&RefField(i + obj.fields.len() - obj.own_fields.len()))
        {
            continue;
        }
        fields.push(ClassField {
            name: f.name(code).to_owned(),
            static_: false,
            ty: f.t,
        });
    }
    if let Some(ty) = static_type {
        for (i, f) in ty.own_fields.iter().enumerate() {
            if ty
                .bindings
                .contains_key(&RefField(i + ty.fields.len() - ty.own_fields.len()))
            {
                continue;
            }
            fields.push(ClassField {
                name: f.name(code).to_owned(),
                static_: true,
                ty: f.t,
            });
        }
    }

    let mut methods = Vec::new();
    let mut function_indexes = Vec::new();

    for fun in obj.bindings.values() {
        methods.push(stub_function(code, fun.as_fn(code).unwrap()));
        function_indexes.push(fun.as_fn(code).unwrap().findex);
    }
    if let Some(ty) = static_type {
        for fun in ty.bindings.values() {
            methods.push(stub_function(code, fun.as_fn(code).unwrap()));
            function_indexes.push(fun.as_fn(code).unwrap().findex);
        }
    }
    for f in &obj.protos {
        methods.push(stub_function(code, f.findex.as_fn(code).unwrap()));
        function_indexes.push(f.findex);
    }

    let class = Class {
        name: obj.name(code).to_owned(),
        parent: obj
            .super_
            .and_then(|ty| ty.as_obj(code))
            .map(|ty| ty.name(code).to_owned()),
        fields,
        methods,
    };

    let function_indexes_usize: Vec<usize> = function_indexes.iter().map(|ref_fun| ref_fun.0).collect();
    (class, function_indexes_usize)
}

#[pymethods]
impl Bytecode {
    #[new]
    fn new(path: String) -> PyResult<Self> {
        catch_panic(|| {
            let bytecode = _Bytecode::from_file(path)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))?;
            Ok(Self { bytecode: Arc::new(bytecode) })
        })        
    }

    fn get_debug_files(&self) -> PyResult<Vec<String>> {
        catch_panic(|| {
            match &self.bytecode.debug_files {
                Some(files) => {
                    let string_files: Vec<String> = files.iter().map(|f| f.to_string()).collect();
                    Ok(string_files)
                },
                None => Ok(Vec::new()),
            }
        })
    }

    fn get_functions(&self, file: String) -> PyResult<Vec<String>> {
        catch_panic(|| {
            let debug_files = self.bytecode.debug_files.as_ref().ok_or_else(|| 
                PyErr::new::<pyo3::exceptions::PyException, _>("No debug files found"))?;
            
            if let Some(idx) = debug_files.iter().enumerate().find_map(|(i, d)| {
                if d == &file {
                    Some(i)
                } else {
                    None
                }
            }) {
                let functions: Vec<String> = self.bytecode.functions.iter().enumerate().filter_map(|(_i, f)| {
                    if let Some(debug_info) = &f.debug_info {
                        if !debug_info.is_empty() && debug_info[debug_info.len() - 1].0 == idx {
                            Some(format!("{}", f.display_header::<EnhancedFmt>(&self.bytecode)))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }).collect();
                
                Ok(functions)
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!("File {} not found!", file)))
            }
        })
    }

    fn decompile(&self, function_idx: String) -> PyResult<String> {
        catch_panic(|| {
            if let Some(fun) = RefFun(function_idx.parse::<usize>().unwrap()).as_fn(&self.bytecode) {
                let formatted = format!(
                    "{}",
                    hlbc_decompiler::decompile_function(&self.bytecode, fun)
                        .display(&self.bytecode, &hlbc_decompiler::fmt::FormatOptions::new(2))
                );
                Ok(formatted)
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!("Function with index {} not found!", function_idx)))
            }
        })
    }

    fn stub(&self, function_idx: String) -> PyResult<String> {
        catch_panic(|| {
            if let Some(fun) = RefFun(function_idx.parse::<usize>().unwrap()).as_fn(&self.bytecode) {
                let formatted = format!(
                    "{}",
                    stub_function(&self.bytecode, fun)
                        .display(&self.bytecode, &hlbc_decompiler::fmt::FormatOptions::new(2))
                );
                Ok(formatted)
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!("Function with index {} not found!", function_idx)))
            }
        })
    }

    fn decompile_class(&self, type_idx: String) -> PyResult<String> {
        catch_panic(|| {
            let idx = type_idx.parse::<usize>().unwrap();
            let ty = &self.bytecode.types[idx];
            match ty {
                Type::Obj(obj) => {
                    Ok(format!(
                        "{}",
                        hlbc_decompiler::decompile_class(&*self.bytecode, obj)
                            .display(&*self.bytecode, &hlbc_decompiler::fmt::FormatOptions::new(2))
                    ))
                }
                _ => Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!("Index {type_idx} is not an object!"))),
            }
        })
    }

    fn stub_class(&self, type_idx: String) -> PyResult<(String, Vec<String>)> {
        catch_panic(|| {
            let idx = type_idx.parse::<usize>().unwrap();
            let ty = &self.bytecode.types[idx];
            match ty {
                Type::Obj(obj) => {
                    let (class, function_indexes) = stub_class(&self.bytecode, obj);
                    let formatted_class = format!("{}", class.display(&self.bytecode, &hlbc_decompiler::fmt::FormatOptions::new(2)));
                    let function_indexes_str = function_indexes.iter().map(|idx| idx.to_string()).collect();
                    Ok((formatted_class, function_indexes_str))
                }
                _ => Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!("Index {type_idx} is not an object!"))),
            }
        })
    }

    fn get_function(&self, function_idx: String) -> PyResult<String> {
        catch_panic(|| {
            if let Some(fun) = RefFun(function_idx.parse::<usize>().unwrap()).as_fn(&self.bytecode) {
                Ok(format!("{}", fun.display::<EnhancedFmt>(&self.bytecode)))
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!("Function with index {} not found!", function_idx)))
            }
        })
    }

    fn get_all_types(&self, include_std: bool) -> PyResult<Vec<String>> {
        catch_panic(|| {
            let classes: Vec<String> = self.bytecode.types.iter().enumerate().filter_map(|(i, t)| {
                if (include_std || !t.is_from_std(&self.bytecode)) && !t.is_wrapper_type() {
                    Some(i.to_string())
                } else {
                    None
                }
            }).collect();
            Ok(classes)
        })
    }

    fn get_class_file(&self, type_idx: String) -> PyResult<String> {
        // get the class, then get the first method and see what file it's in
        // if there are no methods, return "lost_and_found.hx"
        catch_panic(|| {
            let idx = type_idx.parse::<usize>().unwrap();
            let ty = &self.bytecode.types[idx];
            match ty {
                Type::Obj(obj) => {
                    for proto in &obj.protos {
                        if let Some(fun) = proto.findex.as_fn(&self.bytecode) {
                            if let Some(debug_info) = &fun.debug_info {
                                if !debug_info.is_empty() {
                                    let file_idx = debug_info[0].0;
                                    if let Some(debug_files) = &self.bytecode.debug_files {
                                        if let Some(file) = debug_files.get(file_idx) {
                                            return Ok(file.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok("lost_and_found.hx".to_string())
                }
                _ => Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!("Index {type_idx} is not an object!"))),
            }
        })
    }

    fn class_named(&self, name: String) -> PyResult<String> {
        catch_panic(|| {
            let classes: Vec<String> = self.bytecode.types.iter().enumerate().filter_map(|(i, t)| {
                if let Type::Obj(obj) = t {
                    if obj.name(&self.bytecode) == name {
                        Some(i.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).collect();
            if classes.len() == 1 {
                Ok(classes[0].clone())
            } else if classes.len() == 0 {
                Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!("No class named {} found!", name)))
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!("Multiple classes named {} found!", name)))
            }
        })
    }

    fn copy_function_from(&mut self, other: &Self, source_idx: String, dest_idx: String) -> PyResult<()> {
        catch_panic(|| {
            let source = RefFun(source_idx.parse::<usize>().unwrap()).as_fn(&other.bytecode).ok_or_else(|| 
                PyErr::new::<pyo3::exceptions::PyException, _>(format!("Function with index {} not found!", source_idx)))?;
            let mut dest_bytecode = Arc::clone(&self.bytecode);
            let mut dest_bytecode = Arc::make_mut(&mut dest_bytecode);
            dest_bytecode.functions[dest_idx.parse::<usize>().unwrap()] = source.clone();
            self.bytecode = Arc::new(dest_bytecode.clone());
            Ok(())
        })
    }
    
    fn serialise_to(&self, path: String) -> PyResult<()> {
        catch_panic(|| {
            let mut file = std::fs::File::create(path).map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))?;
            self.bytecode.serialize(&mut file).map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))?;
            Ok(())
        })
    }
}

#[pymodule]
fn hlbc(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Bytecode>()?;
    Ok(())
}