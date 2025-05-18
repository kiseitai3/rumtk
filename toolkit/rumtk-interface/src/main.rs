/*
 *     rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 *     This toolkit aims to be reliable, simple, performant, and standards compliant.
 *     Copyright (C) 2025  Luis M. Santos, M.D.
 *
 *     This program is free software: you can redistribute it and/or modify
 *     it under the terms of the GNU General Public License as published by
 *     the Free Software Foundation, either version 3 of the License, or
 *     (at your option) any later version.
 *
 *     This program is distributed in the hope that it will be useful,
 *     but WITHOUT ANY WARRANTY; without even the implied warranty of
 *     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *     GNU General Public License for more details.
 *
 *     You should have received a copy of the GNU General Public License
 *     along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use clap::Parser;
use rumtk_core::core::RUMResult;
use rumtk_core::strings::RUMString;
use rumtk_core::{rumtk_deserialize, rumtk_read_stdin, rumtk_serialize, rumtk_write_stdout};
use rumtk_hl7_v2::hl7_v2_mllp::mllp_v2::{SafeAsyncMLLP, SafeMLLPChannel, MLLP_FILTER_POLICY};
use rumtk_hl7_v2::hl7_v2_parser::v2_parser::V2Message;
use rumtk_hl7_v2::{
    rumtk_v2_generate_message, rumtk_v2_mllp_connect, rumtk_v2_mllp_iter_channels,
    rumtk_v2_mllp_listen,
};

///
/// HL7 V2 Interface CLI
///
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct RUMTKInterfaceArgs {
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
    port: Option<u16>,
    ///
    /// Filter mode under which the interface will operate. Meaning, if an input has unescaped
    /// characters that should have been escaped per the standard, what should the interface do
    /// to handle them.
    ///
    /// Options should be `escape`, `filter`, `none`.
    ///
    /// The program defaults to enforcing escaping the message before going outbound as specified
    /// in the standard.
    ///
    #[arg(short, long, default_value_t = RUMString::from("escape"))]
    filter_policy: RUMString,
    ///
    /// Specifies command line script to execute on message.
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
    /// Is the interface meant to be bound to the loopback address and remain hidden from the
    /// outside world.
    ///
    /// If a NIC IP is defined via `--ip`, that value will override this flag.
    ///
    #[arg(short, long)]
    local: bool,
}

fn outbound_loop(channel: &SafeMLLPChannel) {
    loop {
        let stdin_msg = match rumtk_read_stdin!() {
            Ok(msg) => msg,
            Err(e) => continue, // TODO: missing log call
        };
        if !stdin_msg.is_empty() {
            let msg: V2Message =
                rumtk_deserialize!(&stdin_msg).expect("Failed to deserialize stdin input!");
            let raw_message = rumtk_v2_generate_message!(&msg);
            let mut owned_channel = channel.lock().expect("Failed to lock channel");
            owned_channel.send_message(&raw_message).unwrap();
        }
    }
}

fn inbound_loop(listener: &SafeAsyncMLLP) {
    loop {
        for channel in rumtk_v2_mllp_iter_channels!(&listener) {
            let mut owned_channel = channel.lock().expect("Failed to lock channel");
            let raw_msg = match owned_channel.receive_message() {
                Ok(msg) => msg,
                Err(e) => continue, // TODO: missing log call.
            };
            let msg = V2Message::from_str(&raw_msg);
            let serialized_message = match rumtk_serialize!(&msg) {
                Ok(msg) => msg,
                Err(e) => continue, // TODO: missing log call
            };
            rumtk_write_stdout!(&serialized_message);
        }
    }
}

fn main() {
    let args = RUMTKInterfaceArgs::parse();

    let mllp_filter_policy = match args.filter_policy.as_str() {
        "escape" => MLLP_FILTER_POLICY::ESCAPE_INPUT,
        "filter" => MLLP_FILTER_POLICY::FILTER_INPUT,
        "none" => MLLP_FILTER_POLICY::NONE,
        _ => MLLP_FILTER_POLICY::ESCAPE_INPUT,
    };

    if args.outbound {
        let ip = args.ip.expect("Must provide an IP address");
        let port = args.port.expect("Must provide a port number");
        let client =
            rumtk_v2_mllp_connect!(&ip, port, mllp_filter_policy).expect("MLLP connection failed");
        let channel_option = rumtk_v2_mllp_iter_channels!(&client);
        let channel = channel_option.get(0).expect("MLLP connection failed");
        outbound_loop(&channel);
    } else {
        // Build listener
        let mut listener: RUMResult<SafeAsyncMLLP> = Err(RUMString::new(""));
        if args.ip.is_none() && args.port.is_none() {
            listener = rumtk_v2_mllp_listen!(mllp_filter_policy, args.local);
        } else if args.ip.is_none() && !args.port.is_none() {
            listener = rumtk_v2_mllp_listen!(args.port.unwrap(), mllp_filter_policy, args.local);
        } else if !args.ip.is_none() && !args.port.is_none() {
            listener = rumtk_v2_mllp_listen!(
                &args.ip.unwrap(),
                args.port.unwrap(),
                mllp_filter_policy,
                args.local
            );
        } else {
            listener = rumtk_v2_mllp_listen!(mllp_filter_policy, args.local);
        }
        // Run inbound logic
        inbound_loop(
            &listener.expect("MLLP listening connection failed to bind a network interface!"),
        );
    }
}
