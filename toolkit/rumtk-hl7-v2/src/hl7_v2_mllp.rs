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

///
/// # Release 2 of the Minimal Lower Layer Message Transport protocol (MLLP, a.k.a. MLP)
///
/// # Ch. 1
///
///     The MLLP messaging infrastructure layer, and the message transport layer. Messaging
///     Adapters live inside the Application and provide the interface to the specific messaging
///     stack being used. Messaging adapters are both aware of HL7 and the messaging stack being
///     interfaced. Their role is to prepare the HL7 message for transmission by the messaging
///     infrastructure. The Messaging Infrastructure consists of the runtime components that
///     implement a particular messaging protocol. These components are generally off-the-shelf
///     implementations that have no knowledge of the specific payload being transported. Message
///     Transport: this layer represents the means by which the HL7 message is transported to the
///     appropriate destination. Different protocols might use multiple transports, depending on
///     the implementation, the degree of separation between the protocol and the transport and a
///     number of other factors.
///
/// ## 1.2 - Protocol specification
///
///     The goal of the MLLP Message Transport protocol is to provide an interface between HL7
///     Applications and the transport protocol that uses minimal overhead. MLLP is based on a
///     minimalistic OSI-session layer framing protocol. It is assumed that MLLP will be used only
///     in a network environment. Most of the details of error detection and correction are handled
///     by the lower levels of any reasonable transport protocol (e.g. TCP/IP, SNA) and do not
///     require any supplementation. The network protocol and the network behavior have to be agreed
///     upon by the communicating parties prior to the exchange of data. MLLP Release 2 covers the
///     absolute minimal requirements in order for it to be a reliable Message Transport protocol.
///
///     MLLP has limited support for character encodings, see below for details. MLLP supports
///     (amongst other message encodings and ITSs) the vertical bar and XML HL7 version 2 message
///     encodings and the version 3 XML ITS. It may not be applicable to some HL7 version 3 ITSs.
///     ITS's may require an inherent protocol stack that precludes their use of MLLP.
///
/// ### 1.2.1 - Content exchange model
///
///             HL7 Query/Response
/// Source/ Destination           |                  Destination/Source
/// -------------------                             ------------------
///         |--------------|HL7 Content: Query()|----------->|
///         |<------------|Commit Acknowledgement|-----------|
///         |                                                |
///         |<-----------|HL7 Content: Response()|-----------|
///         |------------|Commit Acknowledgement|----------->|
///
///     All HL7 Content (of any kind or type) is framed in a Block and sent to the Destination
///     system. The Destination system acknowledges the receipt of the Block by returning a Commit
///     Acknowledgement message. If the HL7 Content (a query in the example below) triggers the
///     sending of HL7 Content (a Response) by the Destination system, then this HL7 Content is
///     framed in a Block and sent. MLLP has no knowledge of the HL7 Content, nor does it base any
///     part of its behaviour on HL7 Content.
///
/// The behavior of the Source
/// --------------------------
///
/// 1. "Send Block with HL7 Content, block and wait for Affirmative Commit Acknowledgement, Negative Commit Acknowledge, or a Timeout. "
/// 2. "In case of Affirmative Commit Acknowledgement (ACK), finished. "
/// 3.  If case of Negative Commit Acknowledgement the subsequent step is subject to implementation decisions. The default behavior is
///     1.  If the preset number of retries has been reached, notify sender of delivery failure, with reason code.
///     2.  Otherwise go to step 1 to resend the block.
/// 4.  In case of a Timeout the subsequent step is subject to implementation decisions. The default behavior is:
///     1.  If preset number of retries has been reached, or if a pre-specified time has elapsed, notify SENDER of delivery failure, with reason code.
///     2.  otherwise go to step 1 to resend the Block.
///
/// The behavior of the Destination
/// -------------------------------
///
/// 1.  Receive and ignore any received bytes until the start of a Block is found.
/// 2.  Continue to receive bytes until the end of a Block is found, or until a Timeout occurs.
/// 3.  In case of a Timeout, ignore all bytes received thus far; go to step 1.
/// 4.  Once an entire Block has been received, attempt to commit the HL7 Content to storage.
/// 5.  In case the HL7 Content has been successfully committed to storage, send an Affirmative Commit Acknowledgement (ACK); go to step 1.
/// 6.  In case the HL7 Content can't be committed to storage, send a Negative Commit Acknowledgement (NAK); go to step 1.
///
pub mod mllp_v2 {
    //! ### 1.2.2 - Block Format
    //! #### 1.2.2.1 - HL7 Content Block
    //!
    //! This is the format of a message containing data.
    //!
    //!     HL7-Content-Block = SB, dddd, EB, CR.
    //!         dddd = ( printableChar | CR )-sequence.
    //!         printableChar = 0x20 | 0x21 | 0x22 | .. | 0xFF.
    //!         SB = 0x0B.
    //!         EB = 0x1C.
    //!         CR = 0x0D.
    //!
    //! #### 1.2.2.2 - Commit Acknowledgement Block
    //!
    //! This is the format for a message whose content is a single byte acknowledging or negative-acknowledging
    //!
    //!     Commit-Acknowledgement-Block = SB, ( ACK | NAK ), EB, CR.
    //!         SB = 0x0B.
    //!         ACK = 0x06.
    //!         NAK = 0x15.
    //!         EB = 0x1C.
    //!         CR = 0x0D.
    //!
    //! ### 1.2.3 - Limitations of MLLP
    //!
    //! The MLLP Block is framed by single-byte values. The characters transmitted within the MLLP
    //! Block have to be encoded in such a way that the HL7 Content does not conflict with the byte
    //! values used for framing. Some multi-byte character encodings (e.g. UTF-16, UTF-32) may
    //! result in byte values equal to the MLLP framing characters or byte values lower than 0x1F,
    //! resulting in errors. These character encodings are therefore not supported by MLLP.
    //!
    //! Note on supported encodings (FAQ): MLLP supports all single-byte character encodings
    //! (e.g. iso-8859-x, cp1252) as well as UTF-8 and Shift_JIS. The byte values used by UTF-8 do
    //! not conflict with the byte values used for MLLP framing.
    //!
    //! The sending and receiving systems will have to mutually agree upon the encoding used for a
    //! given connection. Most applications within a certain geographic/language area share the same
    //!  character encoding. U.S./Canadian implementations of MLLP typically use the UTF-8 encoding;
    //! Western European (Germanic and Latin language areas) implementations typically use the ISO
    //! 8859-1 encoding.
    //!
    //! ## 1.3 - Examples
    //! ### 1.3.1 - HL7 version 2 Example
    //!
    //!     <SB>
    //!      MSH|^~\&|ZIS|1^AHospital|||200405141144||ADT^A01|20041104082400|P|2.3|||
    //!      AL|NE|||8859/15|<CR>EVN|A01|20041104082400.0000+0100|20041104082400<CR>
    //!      PID||""|10||Vries^Danny^D.^^de||19951202|M|||Rembrandlaan^7^Leiden^^7301TH^""
    //!      ^^P||""|""||""|||||||""|""<CR>PV1||I|3w^301^""^01|S|||100^van den Berg^^A.S.
    //!      ^^""^dr|""||9||||H||||20041104082400.0000+0100<CR>
    //!     <EB><CR>
    //!
    //! ### 1.3.2 - HL7 version 3 Example
    //!
    //!     <SB>
    //!     <?xml version="1.0" encoding="ISO-8859-15"?>
    //! 	    <MFMT_IN10001NL xmlns="urn:hl7-org:v3" xmlns:voc="urn:hl7-org:v3/voc"
    //! 	            xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    //! 	    <id extension="10213" root="2.16.840.1.113883.2.4.99.1.700222.1"/>
    //! 	    <creationTime value="20050216140000"/>
    //! 	    <versionId>V3ED2005</versionId>
    //! 	    <interactionId extension="MFMT_IN100010NL" root="2.16.840.1.113883"/>
    //! 	    <processingCode code="P"/>
    //! 	    . . .
    //! 	    . . .
    //! 	    </MFMT_IN10001NL>
    //!     <EB><CR>
    //!
    //! ### 1.3.3 - CDA Release 2 Example
    //!
    //!     <SB>
    //!     <?xml version="1.0"?>
    //!         <ClinicalDocument xmlns="urn:hl7-org:v3" xmlns:voc="urn:hl7-org:v3/voc"
    //!         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    //!         xsi:schemaLocation="urn:hl7-org:v3 CDA.ReleaseTwo.Dec.2003.xsd">
    //!
    //! 	    <id extension="c266" root="2.16.840.1.113883.3.933"/>
    //! 	    <code code="11488-4" codeSystem="2.16.840.1.113883.6.1"
    //!                displayName="Consultation note"/>
    //! 	    <title>Good Health Clinic Consultation Note</title>
    //! 	    <effectiveTime value="20040407"/>
    //! 	    <setId extension="BB35" root="2.16.840.1.113883.3.933"/>
    //! 	    <versionNumber value="2"/>
    //! 			    . . .
    //! 		    	. . .
    //!         </ClinicalDocument>
    //!     <EB><CR>
    //!
    //! ### 1.3.4 - MLLP Commit Acknowledgement Example
    //!
    //!     <SB><ACK><EB><CR>
    //!
    //! ### 1.3.5 - MLLP Negative Commit Acknowledgement Example
    //!
    //!     <SB><NAK><EB><CR>
    //!

    use rumtk_core::core::RUMResult;
    use rumtk_core::net::tcp::{
        ClientList, RUMClientHandle, RUMNetMessage, RUMServerHandle, ANYHOST, LOCALHOST,
    };
    use rumtk_core::strings::{
        escape, filter_non_printable_ascii, try_decode, RUMString, ToCompactString,
    };
    use rumtk_core::{rumtk_connect, rumtk_create_server, rumtk_start_server};
    use std::sync::{Arc, LockResult, Mutex, MutexGuard};

    /// Timeouts have to be agreed upon by the communicating parties. It is recommended that the
    /// Source use a timeout of between 5 and 30 seconds before giving up on listening for a Commit
    /// Acknowledgement.
    pub const TIMEOUT_SOURCE: u8 = 30;
    /// Timout step interval between checks for ACK. If we reach [TIMEOUT_SOURCE], give up and mark
    /// no ACK received.
    pub const TIMEOUT_STEP_SOURCE: u8 = 1;
    /// It is recommended that the Destination use a timeout that is at least
    /// twice as high as the Source's timeout (e.g. 40 seconds or more) before flushing its inbound
    /// buffer.
    pub const TIMEOUT_DESTINATION: u8 = 60;
    /// Same as [TIMEOUT_STEP_SOURCE], but with a cut off relative to [TIMEOUT_DESTINATION].
    pub const TIMEOUT_STEP_DESTINATION: u8 = 1;
    /// Start Block character (1 byte). ASCII <VT>, i.e., <0x0B>.
    /// This should not be confused with the ASCII characters SOH or STX.
    pub const SB: u8 = 0x0b;
    /// Acknowledgement character (1 byte, ASCII <ACK>, i.e., <0x06>)
    pub const ACK: u8 = 0x06;
    /// Negative-acknowledgement character (1 byte, ASCII <NAK>, i.e., <0x15>)
    pub const NACK: u8 = 0x15;
    /// End Block character (1 byte). ASCII <FS>, i.e., <0x1C>.
    pub const EB: u8 = 0x1c;
    /// Carriage Return (1 byte). ASCII <CR> character, i.e., <0x0D>.
    pub const CR: u8 = 0x0d;

    ///
    /// Encodes a payload using the message format defined by the HL7 spec.
    ///
    /// *\<[SB]\>payload\<[EB]\>\<[CR]\>*
    ///
    pub fn mllp_encode(message: &RUMString) -> RUMNetMessage {
        let mut packaged = RUMNetMessage::with_capacity(message.len() + 3);
        packaged.push(SB);
        packaged.extend(message.as_bytes());
        packaged.push(EB);
        packaged.push(CR);
        packaged
    }

    ///
    /// Opposite of [mllp_encode]. Strips the incoming message off the \<[SB]\>, \<[EB]\>, and \<[CR]\>.
    /// The remaining data is considered an ASCII or UTF-8 payload. However, to be on the safe side,
    /// we use the [try_decode] function from the strings module to attempt auto-detection of encoding
    /// and forcing the output to be in UTF-8.
    ///
    pub fn mllp_decode(message: &RUMNetMessage) -> RUMString {
        let mut stripped = message.clone();
        stripped.retain(|c| c != &SB && c != &EB && c != &CR);
        try_decode(message)
    }

    ///
    /// Depending on [MLLP_FILTER_POLICY], transform the string payload.
    ///
    /// -   If policy is None => clone string.
    /// -   If policy is escape => force escaping of string input such that it is all within the
    ///      printable range of ASCII.
    /// -   If the policy is to filter, remove all non printable ASCII characters and weird bytes.
    ///
    /// I made this function to allow utilities to better control what kind of outbound message
    /// sanitization to enforce in the production environment.
    ///
    pub fn mllp_filter_message(
        msg: &RUMString,
        mllp_filter_policy: &MLLP_FILTER_POLICY,
    ) -> RUMString {
        match mllp_filter_policy {
            MLLP_FILTER_POLICY::NONE => msg.clone(),
            MLLP_FILTER_POLICY::ESCAPE_INPUT => escape(msg),
            MLLP_FILTER_POLICY::FILTER_INPUT => filter_non_printable_ascii(msg),
        }
    }

    ///
    /// Abstraction wrapper that allows us to treat a server or a client connection as a singular
    /// connection layer such that we can establish a two way single channel communication.
    ///
    pub enum LowerLayer {
        SERVER(RUMServerHandle),
        CLIENT(RUMClientHandle),
    }

    impl LowerLayer {
        pub fn init(ip: &str, port: u16, as_server: bool) -> RUMResult<LowerLayer> {
            match as_server {
                true => {
                    let mut server = rumtk_create_server!(ip, port)?;
                    rumtk_start_server!(&mut server);
                    Ok(LowerLayer::SERVER(server))
                }
                false => {
                    let client = rumtk_connect!(&ip, port)?;
                    Ok(LowerLayer::CLIENT(client))
                }
            }
        }

        pub fn send_message(
            &mut self,
            message: &RUMNetMessage,
            client_id: &RUMString,
        ) -> RUMResult<()> {
            match *self {
                LowerLayer::SERVER(ref mut server) => server.send(&client_id, &message),
                LowerLayer::CLIENT(ref mut client) => client.send(&message),
            }
        }

        pub fn receive_message(&mut self, client_id: &RUMString) -> RUMResult<RUMNetMessage> {
            match *self {
                LowerLayer::SERVER(ref mut server) => server.receive(client_id),
                LowerLayer::CLIENT(ref mut client) => Ok(client.receive()?),
            }
        }

        pub fn get_clients(&self) -> ClientList {
            match *self {
                LowerLayer::SERVER(ref server) => server.get_clients(),
                LowerLayer::CLIENT(ref client) => {
                    vec![client.get_address().expect("No client address!")]
                }
            }
        }

        pub fn get_address_info(&self) -> Option<RUMString> {
            match *self {
                LowerLayer::SERVER(ref server) => server.get_address_info(),
                LowerLayer::CLIENT(ref client) => client.get_address(),
            }
        }
    }

    ///
    /// Enum detailing filter options available during MLLP message encoding.
    ///
    pub enum MLLP_FILTER_POLICY {
        /// Do nothing and send message as is.
        /// This is not the recommended method as it allows for malformed messages to make it to
        /// other potentially non-compliant interfaces and cause issues. In the best case, nothing
        /// happens. In most cases, the receiving application breaks and patient care gets delayed.
        /// No good.
        NONE = 0,
        /// Make sure all non ASCII or non-printable characters are escaped and encoded per HL7 guidelines.
        /// This is the proper way to deal with non HL7 V2 compliant messages generated by applications.
        /// As a result, we provide a way here to enforce compliance. This is the default policy for
        /// RUMTK programs.
        ESCAPE_INPUT = 1,
        /// Remove all non ASCII and all non-printable characters from the input message.
        /// The idea here is to enable any CLI to be able to do this if this is how an environment
        /// deals with non compliant interface messages.
        FILTER_INPUT = 2,
    }

    pub type SafeLowerLayer = Arc<Mutex<LowerLayer>>;
    pub type GuardedLowerLayer<'a> = LockResult<MutexGuard<'a, LowerLayer>>;

    ///
    /// # Minimal Lower Layer Protocol
    ///
    /// This is the struct that defines the MLLP.
    /// It handles proper sanitization and encoding/decoding of HL7 flat messages.
    /// It does not handle parsing of messages.
    /// Parsing is left to [v2_parser_interface::v2_parse_message]. This struct only deals with
    /// the low level encoding.
    ///
    pub struct MLLP {
        transport_layer: SafeLowerLayer,
        filter_policy: MLLP_FILTER_POLICY,
        server: bool,
    }

    impl MLLP {
        ///
        /// Establish an [MLLP] connection on any available network interface.
        ///
        pub fn net(port: u16, filter_policy: MLLP_FILTER_POLICY, server: bool) -> RUMResult<MLLP> {
            Ok(MLLP {
                transport_layer: Arc::new(Mutex::new(LowerLayer::init(ANYHOST, port, server)?)),
                filter_policy,
                server,
            })
        }

        ///
        /// Establish an [MLLP] connection within this machine. It only looks at the localhost address.
        ///
        pub fn local(
            port: u16,
            filter_policy: MLLP_FILTER_POLICY,
            server: bool,
        ) -> RUMResult<MLLP> {
            Ok(MLLP {
                transport_layer: Arc::new(Mutex::new(LowerLayer::init(LOCALHOST, port, server)?)),
                filter_policy,
                server,
            })
        }

        ///
        /// Establish an [MLLP] connection on the specified IP/Host and Port.
        ///
        pub fn new(
            ip: &str,
            port: u16,
            filter_policy: MLLP_FILTER_POLICY,
            server: bool,
        ) -> RUMResult<MLLP> {
            Ok(MLLP {
                transport_layer: Arc::new(Mutex::new(LowerLayer::init(ip, port, server)?)),
                filter_policy,
                server,
            })
        }

        fn next_layer(&self) -> GuardedLowerLayer {
            self.transport_layer.lock()
        }

        pub fn send_message(&mut self, message: &RUMString, endpoint: &RUMString) -> RUMResult<()> {
            let filtered = mllp_filter_message(message, &self.filter_policy);
            let encoded = mllp_encode(&filtered);
            self.next_layer().unwrap().send_message(&encoded, endpoint)
        }

        pub fn receive_message(&mut self, endpoint: &RUMString) -> RUMResult<RUMString> {
            let message = self.next_layer().unwrap().receive_message(endpoint)?;
            Ok(mllp_decode(&message))
        }

        pub fn get_clients(&self) -> ClientList {
            self.next_layer().unwrap().get_clients()
        }

        pub fn is_server(&self) -> bool {
            self.server
        }

        pub fn get_address_info(&self) -> Option<RUMString> {
            let lower_layer = self.next_layer().unwrap();
            lower_layer.get_address_info()
        }
    }

    pub type SafeMLLP = Arc<Mutex<MLLP>>;
    pub type GuardedMLLPLayer<'a> = LockResult<MutexGuard<'a, MLLP>>;

    ///
    /// # Minimal Lower Layer Protocol
    ///
    /// ## Intro
    ///
    /// Using the [MLLP] layer and the [LowerLayer] as the lowest layer, create the concept of a
    /// bidirectional channel such that an application can talk to another.
    ///
    pub struct MLLPChannel {
        channel: SafeMLLP,
        peer: RUMString,
    }

    impl MLLPChannel {
        ///
        /// Create vector iterable using the shared [MLLP] instance to obtain channels to clients.
        ///
        pub fn from_server(mllp_instance: &SafeMLLP) -> Vec<RUMResult<MLLPChannel>> {
            let endpoints = mllp_instance.lock().unwrap().get_clients();
            let mut channels = Vec::<RUMResult<MLLPChannel>>::with_capacity(endpoints.len());
            for endpoint in endpoints.iter() {
                channels.push(MLLPChannel::open(endpoint, mllp_instance));
            }
            channels
        }

        ///
        /// Create vector iterable using the shared [MLLP] instance to obtain a single channel to
        /// the endpoint listening interface.
        ///
        pub fn from_client(mllp_instance: &SafeMLLP) -> Vec<RUMResult<MLLPChannel>> {
            let locked_mllp = match mllp_instance.lock() {
                Ok(mllp) => mllp,
                Err(_) => return vec![Err("Could not lock mllp instance!".to_compact_string())],
            };
            let clients = locked_mllp.get_clients();
            let endpoint = clients.get(0).unwrap();
            vec![MLLPChannel::open(endpoint, mllp_instance)]
        }

        pub fn open(endpoint: &RUMString, mllp_instance: &SafeMLLP) -> RUMResult<MLLPChannel> {
            Ok(MLLPChannel {
                peer: endpoint.clone(),
                channel: Arc::clone(mllp_instance),
            })
        }

        pub fn next_layer(&self) -> GuardedMLLPLayer {
            self.channel.lock()
        }

        pub fn send_message(&mut self, message: &RUMString) -> RUMResult<()> {
            self.next_layer().unwrap().send_message(message, &self.peer)
        }

        pub fn receive_message(&mut self) -> RUMResult<RUMString> {
            self.next_layer().unwrap().receive_message(&self.peer)
        }
    }
}

pub mod mllp_v2_api {
    #[macro_export]
    macro_rules! rumtk_v2_mllp_connect {
        ( $port:expr, $policy:expr ) => {{
            use $crate::hl7_v2_mllp::mllp_v2::MLLP;
            MLLP::local($port, $policy, false)
        }};
        ( $ip:expr, $port:expr, $policy:expr ) => {{
            use $crate::hl7_v2_mllp::mllp_v2::MLLP;
            MLLP::new($ip, $port, $policy, false)
        }};
    }

    #[macro_export]
    macro_rules! rumtk_v2_mllp_listen {
        ( $port:expr, $policy:expr, $local:expr ) => {{
            use $crate::hl7_v2_mllp::mllp_v2::MLLP;
            match $local {
                true => MLLP::local($port, $policy, true),
                false => MLLP::net($port, $policy, true),
            }
        }};
    }

    #[macro_export]
    macro_rules! rumtk_v2_mllp_get_channels {
        ( $mllp:expr ) => {{
            match $mllp.is_server() {
                true => MLLPChannel::from_server($mllp),
                false => MLLPChannel::from_client($mllp),
            }
        }};
    }
}
