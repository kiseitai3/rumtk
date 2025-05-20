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

pub mod cli_utils {
    use crate::core::RUMResult;
    use crate::strings::{format_compact, RUMString, RUMStringConversions};
    use clap::Parser;
    use compact_str::CompactStringExt;
    use std::io::{stdin, Read};
    use std::num::NonZeroU16;

    ///
    /// Example CLI parser that can be used to paste in your binary and adjust as needed.
    ///
    /// Note, this is only an example.
    ///
    #[derive(Parser, Debug)]
    #[command(author, version, about, long_about = None)]
    pub struct RUMTKArgs {
        ///
        /// For interface crate only. Specifies the ip address to connect to.
        ///
        /// In outbound mode, `--ip` and `--port` are required parameters.
        ///
        /// In inbound mode, you can omit either or both parameters.
        ///
        #[arg(short, long)]
        ip: Option<RUMString>,
        ///
        /// For interface crate only. Specifies the port to connect to.
        ///
        /// In outbound mode, `--ip` and `--port` are required parameters.
        ///
        /// In inbound mode, you can omit either or both parameters.
        ///
        #[arg(short, long)]
        port: Option<NonZeroU16>,
        ///
        /// For process crate only. Specifies command line script to execute on message.
        ///
        #[arg(short, long)]
        x: Option<RUMString>,
        ///
        /// Number of processing threads to allocate for this program.
        ///
        #[arg(short, long, default_value_t = 1)]
        threads: usize,
        ///
        /// For interface crate only. Specifies if the interface is in outbound mode.
        ///
        /// In outbound mode, `--ip` and `--port` are required parameters.
        ///
        /// In inbound mode, you can omit either or both parameters.
        ///
        #[arg(short, long)]
        outbound: bool,
        ///
        /// Request program runs in debug mode and log more information.
        ///
        #[arg(short, long, default_value_t = false)]
        debug: bool,
        ///
        /// Request program runs in dry run mode and simulate as many steps as possible but not commit
        /// to a critical non-reversible step.
        ///
        /// For example, if it was meant to write contents to a file, stop before doing so.
        ///
        #[arg(short, long, default_value_t = false)]
        dry_run: bool,
    }

    pub fn read_stdin() -> RUMResult<RUMString> {
        let stdin_handle = stdin();
        let mut locked_stdin_handle = stdin_handle.lock();
        let mut message = String::new();
        match locked_stdin_handle.read_to_string(&mut message) {
            Ok(s) => s,
            Err(e) => {
                return Err(format_compact!("Error reading from STDIN: {}", e));
            }
        };
        Ok(message.to_rumstring())
    }

    pub fn print_license_notice(program: &str, year: &str, author_list: &Vec<&str>) {
        let authors = author_list.join_compact(", ");
        let notice = format_compact!(
            "  {program}  Copyright (C) {year}  {authors}
        This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
        This is free software, and you are welcome to redistribute it
        under certain conditions; type `show c' for details."
        );
        println!("{}", notice);
    }
}

pub mod macros {
    ///
    /// Reads STDIN and unescapes the incoming message.
    /// Return this unescaped message.
    ///
    /// # Example
    /// ```
    /// use rumtk_core::core::RUMResult;
    /// use rumtk_core::strings::RUMString;
    /// use crate::rumtk_core::rumtk_read_stdin;
    ///
    /// fn test_read_stdin() -> RUMResult<RUMString> {
    ///     rumtk_read_stdin!()
    /// }
    ///
    /// match test_read_stdin() {
    ///     Ok(s) => (),
    ///     Err(e) => panic!("Error reading stdin because => {}", e)
    /// }
    /// ```
    ///
    #[macro_export]
    macro_rules! rumtk_read_stdin {
        (  ) => {{
            use $crate::cli::cli_utils::read_stdin;
            use $crate::strings::unescape_string;
            let raw_message = read_stdin().expect("Failed to read stdin");
            unescape_string(&raw_message)
        }};
    }

    ///
    /// Escapes a message and writes it to stdout via the print! macro.
    ///
    /// # Example
    /// ```
    /// use rumtk_core::rumtk_write_stdout;
    ///
    /// rumtk_write_stdout!("I ❤ my wife!");
    /// ```
    ///
    #[macro_export]
    macro_rules! rumtk_write_stdout {
        ( $message:expr ) => {{
            use $crate::strings::escape;
            let escaped_message = escape($message);
            print!("{}", &escaped_message);
        }};
    }

    ///
    /// Prints the mandatory GPL License Notice to terminal!
    ///
    /// # Example
    /// ## Default
    /// ```
    /// use rumtk_core::rumtk_print_license_notice;
    ///
    /// rumtk_print_license_notice!();
    /// ```
    /// ## Program Only
    /// ```
    /// use rumtk_core::rumtk_print_license_notice;
    ///
    /// rumtk_print_license_notice!("RUMTK");
    /// ```
    /// ## Program + Year
    /// ```
    /// use rumtk_core::rumtk_print_license_notice;
    ///
    /// rumtk_print_license_notice!("RUMTK", "2025");
    /// ```
    /// ## Program + Year + Authors
    /// ```
    /// use rumtk_core::rumtk_print_license_notice;
    ///
    /// rumtk_print_license_notice!("RUMTK", "2025", &vec!["Luis M. Santos, M.D."]);
    /// ```
    ///
    #[macro_export]
    macro_rules! rumtk_print_license_notice {
        ( ) => {{
            use $crate::cli::cli_utils::print_license_notice;

            print_license_notice("RUMTK", "2025", &vec!["Luis M. Santos, M.D."]);
        }};
        ( $program:expr ) => {{
            use $crate::cli::cli_utils::print_license_notice;
            print_license_notice(&$program, "2025", &vec!["2025", "Luis M. Santos, M.D."]);
        }};
        ( $program:expr, $year:expr ) => {{
            use $crate::cli::cli_utils::print_license_notice;
            print_license_notice(&$program, &$year, &vec!["Luis M. Santos, M.D."]);
        }};
        ( $program:expr, $year:expr, $authors:expr ) => {{
            use $crate::cli::cli_utils::print_license_notice;
            print_license_notice(&$program, &$year, &$authors);
        }};
    }
}
