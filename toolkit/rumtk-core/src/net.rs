/*
 * rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 * This toolkit aims to be reliable, simple, performant, and standards compliant.
 * Copyright (C) 2024  Luis M. Santos, M.D.
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
pub mod tcp {
    use std::collections::VecDeque;
    use std::marker::Tuple;
    use std::sync::Mutex;
    use ahash::{HashMap, HashMapExt};
    use compact_str::{format_compact, ToCompactString};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use crate::core::RUMResult;
    use crate::strings::{RUMString, RUMStringConversions};
    use crate::threading::threading_functions::get_default_system_thread_count;
    pub use tokio::net::{TcpListener, TcpStream};

    pub type RUMNetMessage = Vec<u8>;

    #[derive(Debug)]
    pub struct RUMClient {
        socket: TcpStream,
    }

    impl RUMClient {
        pub async fn connect(ip: &str, port: u16) -> RUMResult<RUMClient> {
            let addr = format_compact!("{}:{}", ip, port);
            match TcpStream::connect(addr.as_str()).await {
                Ok(socket) => {
                    Ok(RUMClient{socket})
                },
                Err(e) => Err(format_compact!("Unable to connect to {} because {}", &addr.as_str(), &e)),
            }
        }

        pub async fn accept(socket: TcpStream) -> RUMResult<RUMClient> {
            Ok(RUMClient{socket})
        }

        pub async fn send(&mut self, msg: &str) -> RUMResult<()> {
            match self.socket.write_all(msg.as_bytes()).await {
                Ok(_) => Ok(()),
                Err(e) => Err(format_compact!("Unable to send message to {} because {}", &self.socket.local_addr().unwrap().to_compact_string(), &e)),
            }
        }

        pub async fn recv(&mut self) -> RUMResult<RUMNetMessage> {
            let mut msg = RUMNetMessage::new();
            match self.socket.read_to_end(&mut msg).await {
                Ok(_) => Ok(msg),
                Err(e) => Err(format_compact!("Error receiving message from {} because {}", &self.socket.peer_addr().unwrap().to_compact_string(), &e)),
            }
        }

        pub async fn read_ready(&self) -> bool {
            match self.socket.readable().await {
                Ok(_) => true,
                Err(_) => false,
            }
        }

        pub async fn write_ready(&self) -> bool {
            match self.socket.writable().await {
                Ok(_) => true,
                Err(_) => false,
            }
        }
    }

    type SafeQueue<T> = Mutex<VecDeque<T>>;
    type SafeClients = Mutex<Vec<RUMClient>>;

    pub struct RUMServer {
        tcp_listener: TcpListener,
        tx_in: SafeQueue<RUMNetMessage>,
        tx_out: HashMap<RUMString, SafeQueue<RUMString>>,
        clients: SafeClients,
    }

    impl RUMServer {
        pub async fn new(ip: &str, port: u16) -> RUMResult<RUMServer> {
            let addr = format_compact!("{}:{}", ip, port);
            let tcp_listener = match TcpListener::bind(addr.as_str()).await {
                Ok(listener) => listener,
                Err(e) => return Err(format_compact!("Unable to bind to {} because {}", &addr.as_str(), &e)),
            };
            let tx_in = SafeQueue::<RUMNetMessage>::new(VecDeque::new());
            let tx_out = HashMap::<RUMString, SafeQueue<RUMString>>::new();
            let clients = SafeClients::new(Vec::new());
            Ok(RUMServer{tcp_listener, tx_in, tx_out, clients})
        }

        #[tokio::main(flavor = "multi_thread")]
        async fn run(&mut self) -> RUMResult<()> {
            loop {
                self.handle_accept().await;
                self.handle_send().await;
                self.handle_receive().await;
            }
        }

        pub fn start(ip: &str, port: u16) -> RUMResult<(tokio::runtime::Handle, RUMServer)> {
            let handle = tokio::runtime::Handle::current();
            let mut server = handle.block_on(RUMServer::new(ip, port))?;
            let task_handle = handle.spawn(async move || -> RUMResult<()> {
                match server.run().await.unwrap() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format_compact!("Failed to run server because {}", &e)),
                }
            });
            Ok((handle, server))
        }

        async fn handle_accept(&mut self) {
            match self.tcp_listener.accept().await {
                Ok((socket, _)) => {
                    let mut client_list = self.clients.lock().unwrap();
                    client_list.push(match RUMClient::accept(socket) {
                        Ok(client) => client,
                        Err(e) => return (),
                    });
                }
                Err(e) => ()
            }
        }

        async fn handle_send(&mut self) {
            for mut client in self.clients.iter() {
                client.write_ready().await;
                let addr = client.socket.peer_addr().unwrap().to_compact_string();
                let mut queue = self.tx_out[&addr].lock().unwrap();
                for msg in queue.iter() {
                    tokio::spawn(client.send(&msg));
                }
                queue.clear();
            }
        }

        pub fn get_clients(&self) -> &SafeClients {
            &self.clients
        }

        pub fn push_message(&mut self, client_id: &RUMString, msg: &str) {
            if !self.tx_out.contains_key(client_id) {
                let new_queue = SafeQueue::<RUMString>::new(VecDeque::new());
                self.tx_out.insert(client_id.clone(), new_queue);
            }
            let mut queue = &self.tx_out[client_id].lock().unwrap();
            queue.push_back(msg.to_rumstring());
        }

        async fn handle_receive(&mut self) {
            for mut client in self.clients.iter() {
                client.read_ready().await;
                tokio::spawn(async move || {
                    let msg = client.recv().await.unwrap();
                    let mut queue = self.tx_in.lock().unwrap();
                    queue.push_back(msg);
                });
            }
        }

        pub fn pop_message(&mut self) -> Option<RUMNetMessage> {
            self.tx_in.lock().unwrap().pop_front()
        }
    }
}

