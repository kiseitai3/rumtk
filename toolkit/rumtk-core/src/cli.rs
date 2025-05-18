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
use crate::strings::RUMString;
use clap::Parser;
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
