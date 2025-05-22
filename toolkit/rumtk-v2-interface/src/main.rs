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
use rumtk_core::net::tcp::LOCALHOST;
use rumtk_core::strings::RUMString;
use rumtk_core::{rumtk_deserialize, rumtk_read_stdin, rumtk_serialize, rumtk_write_stdout};
use rumtk_hl7_v2::hl7_v2_mllp::mllp_v2::{SafeAsyncMLLP, SafeMLLPChannel, MLLP_FILTER_POLICY};
use rumtk_hl7_v2::hl7_v2_parser::v2_parser::V2Message;
use rumtk_hl7_v2::{
    rumtk_v2_generate_message, rumtk_v2_mllp_connect, rumtk_v2_mllp_iter_channels,
    rumtk_v2_mllp_listen,
};

const HL7_V2_REPEATING_FIELD_MESSAGE: &str =
    "MSH|^~\\&#|NIST EHR^2.16.840.1.113883.3.72.5.22^ISO|NIST EHR Facility^2.16.840.1.113883.3.72.5.23^ISO|NIST Test Lab APP^2.16.840.1.113883.3.72.5.20^ISO|NIST Lab Facility^2.16.840.1.113883.3.72.5.21^ISO|20130211184101-0500||OML^O21^OML_O21|NIST-LOI_9.0_1.1-GU_PRU|T|2.5.1|||AL|AL|||||LOI_Common_Component^LOI BaseProfile^2.16.840.1.113883.9.66^ISO~LOI_GU_Component^LOI GU Profile^2.16.840.1.113883.9.78^ISO~LAB_PRU_Component^LOI PRU Profile^2.16.840.1.113883.9.82^ISO\n
        PID|1||PATID14567^^^NIST MPI&2.16.840.1.113883.3.72.5.30.2&ISO^MR||Hernandez^Maria^^^^^L||19880906|F||2054-5^Black or   African American^HL70005|3248 E  FlorenceAve^^Huntington Park^CA^90255^^H||^^PH^^^323^5825421|||||||||H^Hispanic or Latino^HL70189\n
        ORC|NW|ORD231-1^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO|||||||20130116090021-0800|||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI\n
        OBR|1|ORD231-1^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO||34555-3^Creatinine 24H renal clearance panel^LN^^^^^^CreatinineClearance|||201301151130-0800|201301160912-0800||||||||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI\n
        DG1|1||I10^Essential (primary) hypertension^I10C^^^^^^Hypertension, NOS|||F|||||||||2\n
        DG1|2||O10.93^Unspecified pre-existing hypertension complicating the puerperium^I10C^^^^^^Pregnancy with chronic hypertension|||W|||||||||1\n
        OBX|1|CWE|67471-3^Pregnancy status^LN^1903^Pregnancy status^99USL^2.44^^Isthe patient pregnant?||Y^Yes^HL70136^1^Yes, confirmed less than 12 weeks^99USL^2.5.1^^early pregnancy (pre 12 weeks)||||||O|||20130115|||||||||||||||SCI\n
        OBX|2|NM|3167-4^Volume of   24   hour Urine^LN^1904^Urine Volume of 24 hour collection^99USL^2.44^^Urine Volume 24hour collection||1250|mL^milliliter^UCUM^ml^mililiter^L^1.7^^ml|||||O|||20130116|||||||||||||||SCI\n
        OBX|3|NM|3141-9^Body weight Measured^LN^BWm^Body weight Measured^99USL^2.44^^patient weight measured in kg||59.5|kg^kilogram^UCUM|||||O|||20130116|||||||||||||||SCI\n
        SPM|1|S-2312987-1&NIST EHR&2.16.840.1.113883.3.72.5.24&ISO||276833005^24 hour urine sample (specimen)^SCT^UR24H^24hr Urine^99USL^^^24 hour urine|||||||||||||201301151130-0800^201301160912-0800\n
        SPM|2|S-2312987-2&NIST EHR&2.16.840.1.113883.3.72.5.24&ISO||119297000^Blood Specimen^SCT|||||||||||||201301160912-0800ORC|NW|ORD231-2^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO|||||||20130115102146-0800|||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI\n
        OBR|2|ORD231-2^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO||21482-5^Protein [Mass/volume] in 24 hour Urine^LN^^^^^^24 hour Urine Protein|||201301151130-0800|201301160912-0800||||||||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI\n
        DG1|1||I10^Essential (primary) hypertension^I10C^^^^^^Hypertension, NOS|||F|||||||||2";

const VXU_HL7_V2_MESSAGE: &str =
    "MSH|^~\\&|NISTEHRAPP|NISTEHRFAC|NISTIISAPP|NISTIISFAC|20150625072816.601-0500||VXU^V04^VXU_V04|NIST-IZ-AD-10.1_Send_V04_Z22|P|2.5.1|||ER|AL|||||Z22^CDCPHINVS|NISTEHRFAC|NISTIISFAC\n
        PID|1||21142^^^NIST-MPI-1^MR||Vasquez^Manuel^Diego^^^^L||19470215|M||2106-3^White^CDCREC|227 Park Ave^^Bozeman^MT^59715^USA^P||^PRN^PH^^^406^5555815~^NET^^Manuel.Vasquez@isp.com|||||||||2135-2^Hispanic or Latino^CDCREC||N|1|||||N\n
        PD1|||||||||||01^No reminder/recall^HL70215|N|20150625|||A|20150625|20150625ORC|RE||31165^NIST-AA-IZ-2|||||||7824^Jackson^Lily^Suzanne^^^^^NIST-PI-1^L^^^PRN|||||||NISTEHRFAC^NISTEHRFacility^HL70362\n
        RXA|0|1|20141021||152^Pneumococcal Conjugate, unspecified formulation^CVX|999|||01^Historical Administration^NIP001|||||||||||CP|A";

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
    #[arg(short, long, default_value_t = RUMString::from("none"))]
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
    ///
    /// Only used if in client/outbound mode. Places the interface into a loop constantly looking
    /// for messages in stdin to ship to the connected listening interface.
    ///
    #[arg(short, long)]
    daemon: bool,
}

fn outbound_send(channel: &SafeMLLPChannel) -> RUMResult<()> {
    let stdin_msg = rumtk_read_stdin!()?;
    if !stdin_msg.is_empty() {
        let msg: V2Message = match rumtk_deserialize!(&stdin_msg) {
            Ok(msg) => msg,
            Err(e) => V2Message::try_from_str(&stdin_msg)?,
        };
        let raw_message = rumtk_v2_generate_message!(&msg);
        let mut owned_channel = channel.lock().expect("Failed to lock channel");
        return owned_channel.send_message(&raw_message);
    }
    Ok(())
}

fn outbound_loop(channel: &SafeMLLPChannel) {
    loop {
        match outbound_send(channel) {
            Ok(()) => continue,
            Err(e) => panic!("{}", e), // TODO: missing log call
        };
    }
}

fn inbound_loop(listener: &SafeAsyncMLLP) {
    loop {
        for channel in rumtk_v2_mllp_iter_channels!(&listener) {
            let mut owned_channel = channel.lock().expect("Failed to lock channel");
            let raw_msg = match owned_channel.receive_message() {
                Ok(msg) => msg,
                Err(e) => {
                    //println!("{}", e);
                    continue;
                } // TODO: missing log call.
            };
            let msg = match V2Message::try_from_str(&raw_msg) {
                Ok(msg) => msg,
                Err(e) => panic!("{}", e), // TODO: missing log call.
            };
            let serialized_message = match rumtk_serialize!(&msg) {
                Ok(msg) => msg,
                Err(e) => {
                    //println!("{}", e);
                    continue;
                } // TODO: missing log call.
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
        let ip = match args.local {
            true => args.ip.unwrap_or_else(|| LOCALHOST.parse().unwrap()),
            false => args.ip.expect("Must provide an IP address"),
        };
        let port = args.port.expect("Must provide a port number");
        let client =
            rumtk_v2_mllp_connect!(&ip, port, mllp_filter_policy).expect("MLLP connection failed");
        let channel_option = rumtk_v2_mllp_iter_channels!(&client);
        let channel = channel_option.get(0).expect("MLLP connection failed");

        if args.daemon {
            outbound_loop(&channel);
        } else {
            outbound_send(&channel);
        }
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
