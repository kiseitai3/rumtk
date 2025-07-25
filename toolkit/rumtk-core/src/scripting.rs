/*
 * rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 * This toolkit aims to be reliable, simple, performant, and standards compliant.
 * Copyright (C) 2025  Luis M. Santos, M.D.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */

pub mod python_utils {
    use crate::core::RUMResult;
    use compact_str::format_compact;
    use pyo3::prelude::*;
    use pyo3::types::PyList;

    type RUMPyArgs = Py<PyList>;

    ///
    /// Convert a vector of strings to a Python List of strings.
    ///
    /// ## Example
    ///
    /// ```
    ///     use compact_str::format_compact;
    ///     use crate::rumtk_core::scripting::python_utils::{py_args, py_extract};
    ///
    ///     let expect: Vec<&str> = vec!["a", "1", "2"];
    ///
    ///     let py_obj = py_args(&expect).unwrap();
    ///     let result = py_extract(&py_obj);
    ///     assert_eq!(&result, &expect, format_compact!("Python list does not match the input list!\nGot: {:?}\nExpected: {:?}", &result, &expect));
    /// ```
    ///
    pub fn py_args(arg_list: &Vec<&str>) -> RUMResult<RUMPyArgs> {
        Python::with_gil(|py| -> RUMResult<RUMPyArgs> {
            match PyList::new(py, arg_list){
                Ok(pylist) => Ok(pylist.into()),
                Err(e) => {
                    Err(format_compact!(
                            "Could not convert argument list {:#?} into a Python args list because of {:#?}!",
                            &arg_list,
                            e
                        ))
                }
            }
        })
    }

    pub fn py_extract(pyargs: &RUMPyArgs) -> RUMResult<Vec<String>> {
        Python::with_gil(|py| -> RUMResult<Vec<String>> {
            let py_list: Vec<String> = match pyargs.extract(py) {
                Ok(list) => list,
                Err(e) => {
                    return Err(format_compact!(
                        "Could not extract list from Python args! Reason => {:?}",
                        e
                    ));
                }
            };
            Ok(py_list)
        })
    }
}

pub mod python_macros {
    ///
    /// Turns a hash map into a Python dictionary.
    ///
    /// ## Example
    ///
    /// ### HashMap
    ///
    /// ```
    ///     use ahash::{HashMap, HashMapExt};
    ///     use crate::rumtk_core::rumtk_python_create_args;
    ///     use pyo3::{
    ///                 prelude::*,
    ///                 types::{IntoPyDict, PyDict},
    ///             };
    ///
    ///     let mut kv = HashMap::<&str, &str>::new();
    ///     kv.insert("name", "Hello");
    ///     
    ///     let py_dict = rumtk_python_create_args!(kv).unwrap();
    ///
    ///     assert_eq!(kv.keys(), py_dict.keys().iter().collect::<Vec<String>>::(), "Key mismatch!")
    /// ```
    ///
    #[macro_export]
    macro_rules! rumtk_python_create_args {
        ( $kv:expr ) => {{
            use compact_str::format_compact;
            use pyo3::{
                prelude::*,
                types::{IntoPyDict, PyDict},
            };
            match Python::with_gil(|py| -> PyResult<Bound<'_, PyDict>> { $kv.into_py_dict(py) }) {
                Ok(dict) => Ok(dict),
                Err(err) => Err(format_compact!(
                    "Could not generate a dictionary kwargs structure because {}",
                    err
                )),
            }
        }};
    }

    ///
    /// Load a module text into RAM.
    ///
    /// ## Example
    /// ```
    ///     use std::fs::write;
    ///     use uuid::Uuid;
    ///     use crate::rumtk_core::rumtk_python_load_module;
    ///
    ///     let module_fname = format!("{}_module.py", Uuid::new_v4());
    ///     let module_contents = "print(\"Hello World!\")";
    ///     write(&module_fname, module_contents).expect("Failed to write file!");
    ///
    ///     let module_data = rumtk_python_load_module!(&module_fname).unwrap();
    ///
    ///     assert_eq!(module_contents, module_data, "Loaded wrong data!")
    /// ```
    ///
    #[macro_export]
    macro_rules! rumtk_python_load_module {
        ( $mod_path:expr ) => {{
            use compact_str::format_compact;
            use pyo3::{prelude::*, types::IntoPyDict};
            use std::fs::read_to_string;
            match read_to_string($mod_path) {
                Ok(data) => Ok(data),
                Err(err) => Err(format_compact!(
                    "Could not load Python module contents because {}",
                    err
                )),
            }
        }};
    }
}
