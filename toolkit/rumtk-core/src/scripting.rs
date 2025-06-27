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

pub mod python_macros {
    #[macro_export]
    macro_rules! rumtk_python_create_args {
        ( $kv:expr ) => {{
            use compact_str::format_compact;
            use pyo3::{prelude::*, types::IntoPyDict};
            match Python::with_gil(|py| {
                $kv.into_py_dict(py)?;
            }) {
                Ok(dict) => dict,
                Err(err) => Err(format_compact!(
                    "Could not generate a dictionary kwargs structure because {}",
                    err
                )),
            }
        }};
    }

    #[macro_export]
    macro_rules! rumtk_python_load_module {
        ( $mod_path:expr ) => {{
            use compact_str::format_compact;
            use pyo3::{prelude::*, types::IntoPyDict};
            use std::fs::read_to_string;
            match read_to_string($mod_path) {
                Ok(dict) => dict,
                Err(err) => Err(format_compact!(
                    "Could not load Python module contents because {}",
                    err
                )),
            }
        }};
    }
}
