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
    use std::ops::DerefMut;
    use std::task::Poll;
    use tokio::sync::{Mutex as AsyncMutex, RwLock as AsyncRwLock};
    use std::sync::{Arc, Mutex};
    use ahash::{HashMap, HashMapExt};
    use compact_str::{format_compact, ToCompactString};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use crate::core::RUMResult;
    use crate::strings::{RUMString, RUMStringConversions};
    pub use tokio::net::{TcpListener, TcpStream};
    use crate::queue::queue::{TaskItems, TaskQueue, TaskResult};
    use crate::threading::thread_primitives::SafeTaskArgs;

    pub type RUMNetMessage = Vec<u8>;
    pub type ConnectionInfo = (RUMString, u16);

    #[derive(Debug)]
    pub struct RUMClient {
        socket: TcpStream
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

        pub async fn send(&mut self, msg: &RUMString) -> RUMResult<()> {
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

    type SafeQueue<T> = AsyncMutex<VecDeque<T>>;
    type SafeClients = AsyncMutex<Vec<RUMClient>>;

    pub struct RUMServer {
        tcp_listener: TcpListener,
        tx_in: SafeQueue<RUMNetMessage>,
        tx_out: HashMap<RUMString, SafeQueue<RUMString>>,
        clients: SafeClients,
        stop: bool,
        shutdown_completed: bool,
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
            Ok(RUMServer{tcp_listener, tx_in, tx_out, clients, stop: false, shutdown_completed: false})
        }

        async fn run(&mut self) -> RUMResult<()> {
            while !self.stop {
                self.handle_accept().await;
                self.handle_send().await;
                self.handle_receive().await;
            }
            self.shutdown_completed = true;
            Ok(())
        }

        async fn stop_server(&mut self) -> RUMResult<RUMString> {
            self.stop = true;
            while !self.shutdown_completed {}
            Ok(format_compact!("Server shutdown..."))
        }

        async fn handle_accept(&mut self) {
            match self.tcp_listener.accept().await {
                Ok((socket, _)) => {
                    let mut client_list = self.clients.lock().await;
                    client_list.push(match RUMClient::accept(socket).await {
                        Ok(client) => client,
                        Err(e) => return (),
                    });
                }
                Err(e) => ()
            }
        }

        async fn handle_send(&mut self) {
            let mut client_list = self.clients.lock().await;
            for mut client in client_list.iter_mut() {
                client.write_ready().await;
                let addr = client.socket.peer_addr().unwrap().to_compact_string();
                let mut queue = self.tx_out[&addr].lock().await;
                for msg in queue.iter() {
                    match client.send(&msg).await {
                        Ok(_) => (),
                        Err(e) => return (),
                    }
                }
                queue.clear();
            }
        }

        pub fn get_clients(&self) -> &SafeClients {
            &self.clients
        }

        pub async fn push_message(&mut self, client_id: &RUMString, msg: &RUMString) {
            if !self.tx_out.contains_key(client_id) {
                let new_queue = SafeQueue::<RUMString>::new(VecDeque::new());
                self.tx_out.insert(client_id.clone(), new_queue);
            }
            let mut queue = self.tx_out[client_id].lock().await;
            queue.push_back(msg.to_rumstring());
        }

        async fn handle_receive(&mut self) {
            let mut client_list = self.clients.lock().await;
            for mut client in client_list.iter_mut() {
                client.read_ready().await;
                let msg = client.recv().await.unwrap();
                let mut queue = self.tx_in.lock().await;
                queue.push_back(msg);
            }
        }

        pub async fn pop_message(&mut self) -> Option<RUMNetMessage> {
            let mut queue = self.tx_in.lock().await;
            queue.pop_front()
        }
    }
/*
    pub struct RUMClientHandle {
        client: Arc<AsyncRwLock<RUMClient>>,
    }

    impl RUMClientHandle {
        type SendArgs<'a> = (Arc<AsyncRwLock<RUMClient>>, &'a RUMString);
        type ReceiveArgs<> = Arc<AsyncRwLock<RUMClient>>;

        pub fn new(ip: &str, port: u16) -> RUMResult<RUMClientHandle> {
            let con: ConnectionInfo = (RUMString::from(ip), port);
            let mut client = run_quick_async_as_sync!(RUMClientHandle::new_helper, con)?.pop().unwrap();
            Ok(RUMClientHandle{client: Arc::new(AsyncRwLock::new(client))})
        }

        pub fn send(&mut self, msg: &RUMString) -> RUMResult<()> {
            let mut client_ref = Arc::clone(&self.client);
            run_quick_async_as_sync!(RUMClientHandle::send_helper, (client_ref, msg))
        }

        pub fn receive(&mut self) -> RUMResult<RUMNetMessage> {
            let client_ref = Arc::clone(&self.client);
            run_quick_async_as_sync!(RUMClientHandle::receive_helper, client_ref)
        }

        async fn send_helper(args: &SafeTaskArgs<Self::SendArgs<'_>>) -> RUMResult<()> {
            let lock = args.read().await;
            let (mut client_lock_ref, msg) = lock.get(0).unwrap();
            let mut client_lock = Arc::get_mut(&mut Arc::clone(&mut client_lock_ref)).unwrap();
            let mut client = client_lock.read().await;
            *client.send(msg).await
        }

        async fn receive_helper(args: &SafeTaskArgs<Self::ReceiveArgs>) -> RUMResult<RUMNetMessage> {
            let mut arg_list = Arc::clone(args);
            let &&mut mut client = &arg_list[0];
            client.recv().await
        }

        async fn new_helper(args: &SafeTaskArgs<ConnectionInfo>) -> TaskResult<RUMClient> {
            let owned_args =Arc::clone(args);
            let (ip, port) = match owned_args.get(0) {
                Some((ip, port)) => (ip, port),
                None => return Err(format_compact!("No IP address or port provided for connection!")),
            };
            Ok(vec![RUMClient::connect(ip, *port).await?])
        }
    }

    pub struct RUMServerHandle {
        server: Arc<RUMServer>,
    }

    impl RUMServerHandle {
        type SendArgs<'a, 'b, 'c> = (Arc<RUMServer>, &'b RUMString, &'c RUMString);
        type ReceiveArgs<'a> = Arc<RUMServer>;

        pub fn new(ip: &str, port: u16) -> RUMResult<RUMServerHandle> {
            let con: ConnectionInfo = (RUMString::from(ip), port);
            let server = run_quick_async_as_sync!(RUMServerHandle::new_helper, con)?.pop().unwrap();
            Ok(RUMServerHandle{server: Arc::new(server)})
        }

        pub fn start(&mut self) -> RUMResult<()> {
            run_quick_background_task!(RUMServerHandle::start_helper, (Arc::clone(&mut self.server)));
            Ok(())
        }

        pub fn stop(&mut self) -> RUMResult<RUMString> {
            run_quick_async_as_sync!(RUMServerHandle::stop_helper, (Arc::clone(&mut self.server)))
        }

        pub fn send(&mut self, client_id: &RUMString, msg: &RUMString) -> RUMResult<()> {
            run_quick_async_as_sync!(RUMServerHandle::send_helper, (Arc::clone(&mut self.server), client_id, msg))
        }

        pub fn receive(&mut self) -> Option<RUMNetMessage> {
            run_quick_async_as_sync!(RUMServerHandle::receive_helper, Arc::clone(&mut self.server))
        }

        async fn send_helper(args: &SafeTaskArgs<Self::SendArgs<'_, '_, '_>>) -> RUMResult<()> {
            let mut arg_list =Arc::clone(args);
            let (mut server, client_id, msg) = arg_list.pop().unwrap();
            let mut server_ref = Arc::get_mut(&mut server).unwrap();
            Ok(server_ref.push_message(client_id, msg).await)
        }

        async fn receive_helper(args: &SafeTaskArgs<Self::ReceiveArgs<'_>>) -> Option<RUMNetMessage> {
            let mut arg_list =Arc::clone(args);
            let mut server = arg_list.pop().unwrap();
            let mut server_ref = Arc::get_mut(&mut server).unwrap();
            server_ref.pop_message().await
        }

        async fn start_helper(args: &SafeTaskArgs<Self::ReceiveArgs<'_>>) -> RUMResult<()> {
            let mut arg_list = Arc::clone(args);
            let mut server = arg_list.pop().unwrap();
            let mut server_ref = Arc::get_mut(&mut server).unwrap();
            server_ref.run().await
        }

        async fn stop_helper(args: &SafeTaskArgs<Self::ReceiveArgs<'_>>) -> RUMResult<RUMString> {
            let mut arg_list = Arc::clone(args);
            let mut server = arg_list.pop().unwrap();
            let mut server_ref = Arc::get_mut(&mut server).unwrap();
            server_ref.stop_server().await
        }

        async fn new_helper(args: &SafeTaskArgs<ConnectionInfo>) -> TaskResult<RUMServer> {
            let owned_args = Arc::clone(args);
            let (ip, port) = match owned_args.get(0) {
                Some((ip, port)) => (ip, port),
                None => return Err(format_compact!("No IP address or port provided for connection!")),
            };
            Ok(vec![RUMServer::new(ip, *port).await?])
        }
    }
*/
}

pub mod tcp_macros {
    /*
    #[macro_export]
    macro_rules! create_server {
        ( $ip:expr, $port:expr ) => {{
            use $crate::net::tcp::{RUMServerHandle};
            RUMServerHandle::new($ip, $port)
        }};
    }

     */
}

