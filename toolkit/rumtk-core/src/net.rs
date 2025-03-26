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

///
/// This module provides the basic types necessary to be able to handle connections and message
/// transmission in both synchronous and asynchronous contexts.
///
/// The types here should simplify implementation of higher level layers and protocols.
///
pub mod tcp {
    use crate::core::RUMResult;
    use crate::strings::{RUMArrayConversions, RUMString};
    use crate::threading::thread_primitives::{SafeTaskArgs, SafeTokioRuntime, TaskResult};
    use crate::threading::threading_functions::get_default_system_thread_count;
    use crate::{
        rumtk_async_sleep, rumtk_create_task, rumtk_create_task_args, rumtk_init_threads,
        rumtk_resolve_task, rumtk_spawn_task, rumtk_wait_on_task,
    };
    use ahash::{HashMap, HashMapExt};
    use compact_str::{format_compact, ToCompactString};
    use std::collections::VecDeque;
    use std::sync::Arc;
    use tokio::io;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    pub use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::{Mutex as AsyncMutex, RwLock as AsyncRwLock};

    const MESSAGE_BUFFER_SIZE: usize = 1024;

    /// Convenience constant to localhost
    pub const LOCALHOST: &str = "127.0.0.1";
    /// Convenience constant for the `0.0.0.0` address. This is to be used in contexts in which you do not have any interface preference.
    pub const ANYHOST: &str = "0.0.0.0";

    pub type RUMNetMessage = Vec<u8>;
    pub type ReceivedRUMNetMessage = (RUMString, RUMNetMessage);
    type RUMNetPartialMessage = (RUMNetMessage, bool);
    pub type ConnectionInfo = (RUMString, u16);

    ///
    /// This structs encapsulates the [tokio::net::TcpStream] instance that will be our adapter
    /// for connecting and sending messages to a peer or server.
    ///
    #[derive(Debug)]
    pub struct RUMClient {
        socket: TcpStream,
    }

    impl RUMClient {
        ///
        /// Connect to peer and construct the client.
        ///
        pub async fn connect(ip: &str, port: u16) -> RUMResult<RUMClient> {
            let addr = format_compact!("{}:{}", ip, port);
            match TcpStream::connect(addr.as_str()).await {
                Ok(socket) => Ok(RUMClient { socket }),
                Err(e) => Err(format_compact!(
                    "Unable to connect to {} because {}",
                    &addr.as_str(),
                    &e
                )),
            }
        }

        ///
        /// If a connection was already pre-established elsewhere, construct our client with the
        /// connected socket.
        ///
        pub async fn accept(socket: TcpStream) -> RUMResult<RUMClient> {
            Ok(RUMClient { socket })
        }

        ///
        /// Send message to server.
        ///
        pub async fn send(&mut self, msg: &RUMNetMessage) -> RUMResult<()> {
            println!(
                "Sending message to {}: {}",
                self.get_address(false).await.unwrap(),
                msg.to_rumstring()
            );
            match self.socket.write_all(msg.as_slice()).await {
                Ok(_) => Ok(()),
                Err(e) => Err(format_compact!(
                    "Unable to send message to {} because {}",
                    &self.socket.local_addr().unwrap().to_compact_string(),
                    &e
                )),
            }
        }

        ///
        /// Receive message from server. This method will make calls to [RUMClient::recv_some]
        /// indefinitely until we have the full message or stop receiving any data.
        ///
        pub async fn recv(&mut self) -> RUMResult<RUMNetMessage> {
            let mut msg = RUMNetMessage::new();
            loop {
                let mut fragment = self.recv_some().await?;
                msg.append(&mut fragment.0);
                if fragment.1 == false {
                    break;
                }
            }
            if msg.len() > 0 {
                println!(
                    "Received message from {}: {}",
                    self.get_address(false).await.unwrap(),
                    msg.to_rumstring()
                );
            }
            Ok(msg)
        }

        async fn recv_some(&mut self) -> RUMResult<RUMNetPartialMessage> {
            let mut buf: [u8; MESSAGE_BUFFER_SIZE] = [0; MESSAGE_BUFFER_SIZE];
            let client_id = &self.socket.peer_addr().unwrap().to_compact_string();
            match self.socket.try_read(&mut buf) {
                Ok(n) => match n {
                    0 => Err(format_compact!(
                        "Received 0 bytes from {}! It might have disconnected!",
                        &client_id
                    )),
                    MESSAGE_BUFFER_SIZE => Ok((RUMNetMessage::from(buf), true)),
                    _ => Ok((RUMNetMessage::from(buf[0..n].to_vec()), false)),
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    Ok((RUMNetMessage::from(buf), false))
                }
                Err(e) => Err(format_compact!(
                    "Error receiving message from {} because {}",
                    &client_id,
                    &e
                )),
            }
        }

        /// Check if socket is ready for reading.
        pub async fn read_ready(&self) -> bool {
            match self.socket.readable().await {
                Ok(_) => true,
                Err(_) => false,
            }
        }

        /// Check if socket is ready for writing.
        pub async fn write_ready(&self) -> bool {
            match self.socket.writable().await {
                Ok(_) => true,
                Err(_) => false,
            }
        }

        /// Returns the peer address:port as a string.
        pub async fn get_address(&self, local: bool) -> Option<RUMString> {
            match local {
                true => match self.socket.local_addr() {
                    Ok(addr) => Some(addr.to_compact_string()),
                    Err(_) => None,
                },
                false => match self.socket.peer_addr() {
                    Ok(addr) => Some(addr.to_compact_string()),
                    Err(_) => None,
                },
            }
        }
    }

    /// List of client IDs that you can interact with.
    pub type ClientList = Vec<RUMString>;
    type SafeQueue<T> = Arc<AsyncMutex<VecDeque<T>>>;
    pub type SafeClient = Arc<AsyncRwLock<RUMClient>>;
    type SafeClients = Arc<AsyncRwLock<HashMap<RUMString, SafeClient>>>;
    type SafeMappedQueues = Arc<AsyncMutex<HashMap<RUMString, SafeQueue<RUMNetMessage>>>>;
    pub type SafeListener = Arc<AsyncMutex<TcpListener>>;
    pub type SafeServer = Arc<AsyncRwLock<RUMServer>>;

    ///
    /// Enum used for selecting which clients to iterate through.
    /// Pass [SOCKET_READINESS_TYPE::NONE] to ignore filtering by readiness type.
    ///
    pub enum SOCKET_READINESS_TYPE {
        NONE,
        READ_READY,
        WRITE_READY,
        READWRITE_READY,
    }

    ///
    /// This is the Server primitive that listens for incoming connections and manages "low-level"
    /// messages.
    ///
    /// This struct tracks accepting new clients via [RUMServer::handle_accept], incoming messages
    /// via [RUMServer::handle_receive] and message dispatchs via [RUMServer::handle_send].
    ///
    /// All key methods are async and shall be run exclusively in the async context. We provide a
    /// set of tools that allow you to interact with this struct from sync code. One such tool is
    /// [RUMServerHandle].
    ///
    /// The [RUMServer::run] method orchestrate a series of steps that allows starting server
    /// management. The result is that the server will check for connections and messages
    /// autonomously. You want to call this method in a non blocking manner from the sync context,
    /// so that the server can handle the transactions in the background
    ///
    pub struct RUMServer {
        tcp_listener: SafeListener,
        tx_in: SafeMappedQueues,
        tx_out: SafeMappedQueues,
        clients: SafeClients,
        address: Option<RUMString>,
        stop: bool,
        shutdown_completed: bool,
    }

    impl RUMServer {
        ///
        /// Constructs a server and binds the `port` on interface denoted by `ip`. The server
        /// management is not started until you invoke [RUMServer::run].
        ///
        pub async fn new(ip: &str, port: u16) -> RUMResult<RUMServer> {
            let addr = format_compact!("{}:{}", ip, port);
            let tcp_listener_handle = match TcpListener::bind(addr.as_str()).await {
                Ok(listener) => listener,
                Err(e) => {
                    return Err(format_compact!(
                        "Unable to bind to {} because {}",
                        &addr.as_str(),
                        &e
                    ))
                }
            };
            let address = match tcp_listener_handle.local_addr() {
                Ok(addr) => Some(addr.to_compact_string()),
                Err(e) => None,
            };
            let tx_in = SafeMappedQueues::new(AsyncMutex::new(HashMap::<
                RUMString,
                SafeQueue<RUMNetMessage>,
            >::new()));
            let tx_out = SafeMappedQueues::new(AsyncMutex::new(HashMap::<
                RUMString,
                SafeQueue<RUMNetMessage>,
            >::new()));
            let client_list = HashMap::<RUMString, SafeClient>::new();
            let clients = SafeClients::new(AsyncRwLock::new(client_list));
            let tcp_listener = Arc::new(AsyncMutex::new(tcp_listener_handle));
            Ok(RUMServer {
                tcp_listener,
                tx_in,
                tx_out,
                clients,
                address,
                stop: false,
                shutdown_completed: false,
            })
        }

        ///
        /// Main, juicy server management logic. Call this method to kick start a series of
        /// autonomous checks. Message handling and connection handling are taken care
        /// autonamtically.
        ///
        /// Await this method if you wish to block the context thread indefinitely. This method has
        /// a never ending loop looking for when the server has been signalled to shut down.
        ///
        /// `ctx` here refers to an instance of the server wrapped by [SafeServer]. This was done
        /// to be able to make the management logic work autonomously across threads. We call the
        /// RWLock's read() method every pass of the loop and await on it. This allows the runtime
        /// to progress the state of other futures and allow sync code to interact with the server
        /// state. In most situations, this step should yield a no-op.
        ///
        async fn run(ctx: &SafeServer) -> RUMResult<()> {
            // Bootstrapping the main server loop.
            let mut reowned_self = ctx.read().await;
            let mut accept_handle = tokio::spawn(RUMServer::handle_accept(
                Arc::clone(&reowned_self.tcp_listener),
                Arc::clone(&reowned_self.clients),
            ));
            let mut send_handle = tokio::spawn(RUMServer::handle_send(
                Arc::clone(&reowned_self.clients),
                Arc::clone(&reowned_self.tx_out),
            ));
            let mut receive_handle = tokio::spawn(RUMServer::handle_receive(
                Arc::clone(&reowned_self.clients),
                Arc::clone(&reowned_self.tx_in),
            ));
            let mut stop = reowned_self.stop;
            //Most drop here to allow the outside world to grab access to the server handle and interact with us.
            std::mem::drop(reowned_self); //Bootstrap magic that let's the outside able to interact with our server while it runs autonomously in the background.
                                          // Essentially, repeat the above but inside a scope thus automatically freeing the handle to outside access on a routine basis.
            while !stop {
                let mut reowned_self = ctx.read().await;
                if accept_handle.is_finished() {
                    accept_handle = tokio::spawn(RUMServer::handle_accept(
                        Arc::clone(&reowned_self.tcp_listener),
                        Arc::clone(&reowned_self.clients),
                    ));
                }
                if send_handle.is_finished() {
                    send_handle = tokio::spawn(RUMServer::handle_send(
                        Arc::clone(&reowned_self.clients),
                        Arc::clone(&reowned_self.tx_out),
                    ));
                }
                if receive_handle.is_finished() {
                    receive_handle = tokio::spawn(RUMServer::handle_receive(
                        Arc::clone(&reowned_self.clients),
                        Arc::clone(&reowned_self.tx_in),
                    ));
                }
                stop = reowned_self.stop;
            }
            println!("Shutting down server!");
            while !send_handle.is_finished() || !receive_handle.is_finished() {
                rumtk_async_sleep!(0.001).await;
            }
            // Cleanup; signal to the outside world we did finished shutting down and exit execution.
            let mut reowned_self = ctx.write().await;
            reowned_self.shutdown_completed = true;
            println!("Server successfully shut down!");
            Ok(())
        }

        ///
        /// This method signals the server to stop.
        ///
        /// Then, this method waits for run to cleanup before exiting.
        /// Meaning, this method's exit is enough to signal everything went through smoothly.
        ///
        async fn stop_server(ctx: &SafeServer) -> RUMResult<RUMString> {
            println!("Attempting to stop server!");
            let mut reowned_self = ctx.write().await;
            let mut shutdown_completed = reowned_self.shutdown_completed;
            reowned_self.stop = true;
            std::mem::drop(reowned_self);

            // Same trick as run's. We can now opportunistically check if the server exited while
            // safely holding the calling thread hostage.
            while !shutdown_completed {
                rumtk_async_sleep!(0.001).await;
                let mut reowned_self = ctx.read().await;
                shutdown_completed = reowned_self.shutdown_completed;
            }

            Ok(format_compact!("Server fully shutdown!"))
        }

        ///
        /// Contains basic logic for listening for incoming connections.
        ///
        async fn handle_accept(listener: SafeListener, clients: SafeClients) -> RUMResult<()> {
            let server = listener.lock().await;
            match server.accept().await {
                Ok((socket, _)) => {
                    let client = RUMClient::accept(socket).await?;
                    let client_id = match client.get_address(false).await {
                        Some(client_id) => client_id,
                        None => return Err(format_compact!("Accepted client returned no peer address. This should not be happening!"))
                    };
                    let mut client_list = clients.write().await;
                    client_list.insert(client_id, SafeClient::new(AsyncRwLock::new(client)));
                    Ok(())
                }
                Err(e) => Err(format_compact!(
                    "Error accepting incoming client! Error: {}",
                    e
                )),
            }
        }

        ///
        /// Contains logic for sending messages queued for a client to it. `tx_out` is a reference
        /// of [SafeMappedQueues] which is a hash map of [SafeQueue<RUMNetMessage>] whose keys are
        /// the client's peer address string.
        ///
        async fn handle_send(clients: SafeClients, tx_out: SafeMappedQueues) -> RUMResult<()> {
            let mut client_list =
                RUMServer::get_client_ids(&clients, SOCKET_READINESS_TYPE::WRITE_READY).await;
            for client in client_list.iter() {
                let messages = match RUMServer::pop_queue(&tx_out, client).await {
                    Some(messages) => messages,
                    None => continue,
                };
                for msg in messages.iter() {
                    RUMServer::send(&clients, client, msg).await?;
                }
            }
            Ok(())
        }

        ///
        /// Contains the logic for handling receiving messages from clients. Incoming messages are
        /// all placed into a queue that the "outside" world can interact with.
        ///
        async fn handle_receive(clients: SafeClients, tx_in: SafeMappedQueues) -> RUMResult<()> {
            let mut client_list =
                RUMServer::get_client_ids(&clients, SOCKET_READINESS_TYPE::READ_READY).await;
            for client in client_list.iter_mut() {
                let msg = RUMServer::receive(&clients, client).await?;
                RUMServer::push_queue(&tx_in, client, msg).await;
            }
            Ok(())
        }

        pub async fn push_queue(
            tx_queues: &SafeMappedQueues,
            client: &RUMString,
            msg: RUMNetMessage,
        ) {
            let mut queues = tx_queues.lock().await;
            let mut queue = match queues.get_mut(client) {
                Some(queue) => queue,
                None => {
                    let new_queue =
                        SafeQueue::<RUMNetMessage>::new(AsyncMutex::new(VecDeque::new()));
                    queues.insert(client.clone(), new_queue);
                    queues.get_mut(client).unwrap()
                }
            };
            let mut locked_queue = queue.lock().await;
            locked_queue.push_back(msg);
        }

        pub async fn pop_queue(
            tx_queues: &SafeMappedQueues,
            client: &RUMString,
        ) -> Option<Vec<RUMNetMessage>> {
            let mut queues = tx_queues.lock().await;
            let mut queue = match queues.get_mut(client) {
                Some(queue) => queue,
                None => return None,
            };
            let mut locked_queue = queue.lock().await;
            let mut messages = Vec::<RUMNetMessage>::with_capacity(locked_queue.len());
            while !locked_queue.is_empty() {
                let message = match locked_queue.pop_front() {
                    Some(message) => message,
                    None => break,
                };
                messages.push(message);
            }
            locked_queue.clear();
            Some(messages)
        }

        pub async fn send(
            clients: &SafeClients,
            client: &RUMString,
            msg: &RUMNetMessage,
        ) -> RUMResult<()> {
            let owned_clients = clients.read().await;
            match owned_clients.get(client) {
                Some(connected_client) => connected_client.write().await.send(msg).await,
                _ => Err(format_compact!(
                    "Failed to send data! No client with address {} found!",
                    client
                )),
            }
        }

        pub async fn receive(
            clients: &SafeClients,
            client: &RUMString,
        ) -> RUMResult<RUMNetMessage> {
            let owned_clients = clients.read().await;
            match owned_clients.get(client) {
                Some(connected_client) => connected_client.write().await.recv().await,
                _ => Err(format_compact!(
                    "Failed to receive data! No client with address {} found!",
                    client
                )),
            }
        }

        pub async fn get_client_ids(
            clients: &SafeClients,
            ready_type: SOCKET_READINESS_TYPE,
        ) -> ClientList {
            let clients = clients.read().await;
            let mut client_ids = ClientList::with_capacity(clients.len());
            for (client_id, client) in clients.iter() {
                let ready = RUMServer::get_client_readiness(client, &ready_type).await;
                if ready {
                    client_ids.push(RUMServer::get_client_id(client).await);
                }
            }
            client_ids
        }

        pub async fn get_client_id(client: &SafeClient) -> RUMString {
            client
                .read()
                .await
                .get_address(false)
                .await
                .expect("No address found! Malformed client")
        }

        pub async fn get_client_readiness(
            client: &SafeClient,
            socket_readiness_type: &SOCKET_READINESS_TYPE,
        ) -> bool {
            match socket_readiness_type {
                SOCKET_READINESS_TYPE::NONE => true,
                SOCKET_READINESS_TYPE::READ_READY => client.read().await.read_ready().await,
                SOCKET_READINESS_TYPE::WRITE_READY => client.read().await.write_ready().await,
                SOCKET_READINESS_TYPE::READWRITE_READY => {
                    client.read().await.read_ready().await
                        && client.read().await.write_ready().await
                }
            }
        }

        ///
        /// Return list of clients.
        ///
        pub async fn get_clients(&self) -> ClientList {
            RUMServer::get_client_ids(&self.clients, SOCKET_READINESS_TYPE::NONE).await
        }

        ///
        /// Queues a message onto the server to send to client.
        ///
        pub async fn push_message(&mut self, client_id: &RUMString, msg: RUMNetMessage) {
            let mut queue = self.tx_out.lock().await;
            if !queue.contains_key(client_id) {
                let new_queue = SafeQueue::<RUMNetMessage>::new(AsyncMutex::new(VecDeque::new()));
                queue.insert(client_id.clone(), new_queue);
            }
            let mut queue = queue[client_id].lock().await;
            queue.push_back(msg);
        }

        ///
        /// Obtain a message, if available, from the incoming queue.
        ///
        pub async fn pop_message(&mut self, client_id: &RUMString) -> Option<RUMNetMessage> {
            let mut queues = self.tx_in.lock().await;
            let mut queue = match queues.get_mut(client_id) {
                Some(queue) => queue,
                None => return Some(vec![]),
            };
            let mut locked_queue = queue.lock().await;
            locked_queue.pop_front()
        }

        ///
        /// Get the Address:Port info for this socket.
        ///
        pub async fn get_address_info(&self) -> Option<RUMString> {
            self.address.clone()
        }
    }

    ///
    /// Handle struct containing a reference to the global Tokio runtime and an instance of
    /// [SafeClient]. This handle allows sync codebases to interact with the async primitives built
    /// on top of Tokio. Specifically, this handle allows wrapping of the async connect, send, and
    /// receive methods implemented in [RUMClient].
    ///
    pub struct RUMClientHandle {
        runtime: &'static SafeTokioRuntime,
        client: SafeClient,
    }

    impl RUMClientHandle {
        type SendArgs<'a> = (SafeClient, &'a RUMNetMessage);
        type ReceiveArgs = SafeClient;

        pub fn connect(ip: &str, port: u16) -> RUMResult<RUMClientHandle> {
            RUMClientHandle::new(ip, port)
        }

        pub fn new(ip: &str, port: u16) -> RUMResult<RUMClientHandle> {
            let runtime = rumtk_init_threads!(&1);
            let con: ConnectionInfo = (RUMString::from(ip), port);
            let args = rumtk_create_task_args!(con);
            let client = rumtk_wait_on_task!(&runtime, RUMClientHandle::new_helper, &args)?
                .pop()
                .unwrap();
            Ok(RUMClientHandle {
                client: SafeClient::new(AsyncRwLock::new(client)),
                runtime,
            })
        }

        ///
        /// Queues a message send via the tokio runtime.
        ///
        pub fn send(&mut self, msg: &RUMNetMessage) -> RUMResult<()> {
            let mut client_ref = Arc::clone(&self.client);
            let args = rumtk_create_task_args!((client_ref, msg));
            rumtk_wait_on_task!(&self.runtime, RUMClientHandle::send_helper, &args)
        }

        ///
        /// Checks if there are any messages received by the [RUMClient] via the tokio runtime.
        ///
        pub fn receive(&mut self) -> RUMResult<RUMNetMessage> {
            let client_ref = Arc::clone(&self.client);
            let args = rumtk_create_task_args!(client_ref);
            rumtk_wait_on_task!(&self.runtime, RUMClientHandle::receive_helper, &args)
        }

        /// Returns the peer address:port as a string.
        pub fn get_address(&self) -> Option<RUMString> {
            let client_ref = Arc::clone(&self.client);
            let args = rumtk_create_task_args!(client_ref);
            rumtk_wait_on_task!(&self.runtime, RUMClientHandle::get_address_helper, &args)
        }

        async fn send_helper(args: &SafeTaskArgs<Self::SendArgs<'_>>) -> RUMResult<()> {
            let owned_args = Arc::clone(args).clone();
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let (client_lock_ref, msg) = locked_args.get(0).unwrap();
            let mut client_ref = Arc::clone(client_lock_ref);
            let mut client = client_ref.write().await;
            client.send(msg).await
        }

        async fn receive_helper(
            args: &SafeTaskArgs<Self::ReceiveArgs>,
        ) -> RUMResult<RUMNetMessage> {
            let owned_args = Arc::clone(args).clone();
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let mut client_ref = locked_args.get(0).unwrap();
            let mut client = client_ref.write().await;
            client.recv().await
        }

        async fn new_helper(args: &SafeTaskArgs<ConnectionInfo>) -> TaskResult<RUMClient> {
            let owned_args = Arc::clone(args);
            let lock_future = owned_args.read().await;
            let (ip, port) = match lock_future.get(0) {
                Some((ip, port)) => (ip, port),
                None => {
                    return Err(format_compact!(
                        "No IP address or port provided for connection!"
                    ))
                }
            };
            Ok(vec![RUMClient::connect(ip, *port).await?])
        }
        async fn get_address_helper(args: &SafeTaskArgs<Self::ReceiveArgs>) -> Option<RUMString> {
            let owned_args = Arc::clone(args).clone();
            let locked_args = owned_args.read().await;
            let client_ref = locked_args.get(0).unwrap();
            let mut client = client_ref.read().await;
            client.get_address(true).await
        }
    }

    ///
    /// Handle struct containing a reference to the global Tokio runtime and an instance of
    /// [SafeServer]. This handle allows sync codebases to interact with the async primitives built
    /// on top of Tokio. Specifically, this handle allows wrapping of the async bind, send,
    /// receive, and start methods implemented in [RUMServer]. In addition, this handle allows
    /// spinning a server in a fully non-blocking manner. Meaning, you can call start, which will
    /// immediately return after queueing the task in the tokio queue. You can then query the server
    /// for incoming data or submit your own data while the server is operating in the background.
    /// The server can be handling incoming data at the "same" time you are trying to queue your
    /// own message.
    ///
    pub struct RUMServerHandle {
        runtime: &'static SafeTokioRuntime,
        server: SafeServer,
    }

    impl RUMServerHandle {
        type SendArgs = (SafeServer, RUMString, RUMNetMessage);
        type ReceiveArgs = (SafeServer, RUMString);
        type SelfArgs = SafeServer;

        ///
        /// Constructs a [RUMServerHandle] using the detected number of parallel units/threads on
        /// this machine. This method automatically binds to IP 0.0.0.0. Meaning, your server may
        /// become visible to the outside world.
        ///
        pub fn default(port: u16) -> RUMResult<RUMServerHandle> {
            RUMServerHandle::new(ANYHOST, port, get_default_system_thread_count())
        }

        ///
        /// Constructs a [RUMServerHandle] using the detected number of parallel units/threads on
        /// this machine. This method automatically binds to **localhost**. Meaning, your server
        /// remains private in your machine.
        ///
        pub fn default_local(port: u16) -> RUMResult<RUMServerHandle> {
            RUMServerHandle::new(LOCALHOST, port, get_default_system_thread_count())
        }

        ///
        /// General purpose constructor for [RUMServerHandle]. It takes an ip and port and binds it.
        /// You can also control how many threads are spawned under the hood for this server handle.
        ///
        pub fn new(ip: &str, port: u16, threads: usize) -> RUMResult<RUMServerHandle> {
            let runtime = rumtk_init_threads!(&threads);
            let con: ConnectionInfo = (RUMString::from(ip), port);
            let args = rumtk_create_task_args!(con);
            let server = rumtk_wait_on_task!(&runtime, RUMServerHandle::new_helper, &args)?
                .pop()
                .unwrap();
            Ok(RUMServerHandle {
                server: Arc::new(AsyncRwLock::new(server)),
                runtime,
            })
        }

        ///
        /// Starts the main processing loop for the server. This processing loop listens for new
        /// clients in a non-blocking manner and checks for incoming data and data that must be
        /// shipped to clients. You can start the server in a blocking and non_blocking manner.
        ///
        pub fn start(&mut self, blocking: bool) -> RUMResult<()> {
            let args = rumtk_create_task_args!(Arc::clone(&mut self.server));
            let task = rumtk_create_task!(RUMServerHandle::start_helper, args);
            if blocking {
                rumtk_resolve_task!(&self.runtime, task);
            } else {
                rumtk_spawn_task!(&self.runtime, task);
            }
            Ok(())
        }

        ///
        /// Sync API method for signalling the server to stop operations.
        ///
        pub fn stop(&mut self) -> RUMResult<RUMString> {
            let args = rumtk_create_task_args!(Arc::clone(&mut self.server));
            rumtk_wait_on_task!(&self.runtime, RUMServerHandle::stop_helper, &args)
        }

        ///
        /// Sync API method for queueing a message to send a client on the server.
        ///
        pub fn send(&mut self, client_id: &RUMString, msg: &RUMNetMessage) -> RUMResult<()> {
            let args = rumtk_create_task_args!((
                Arc::clone(&mut self.server),
                client_id.clone(),
                msg.clone()
            ));
            let task = rumtk_create_task!(RUMServerHandle::send_helper, args);
            rumtk_resolve_task!(&self.runtime, rumtk_spawn_task!(&self.runtime, task))
        }

        ///
        /// Sync API method for obtaining a single message from the server's incoming queue.
        /// Returns the next available [RUMNetMessage]
        ///
        pub fn receive(&mut self, client_id: &RUMString) -> RUMResult<RUMNetMessage> {
            let args = rumtk_create_task_args!((Arc::clone(&mut self.server), client_id.clone()));
            let task = rumtk_create_task!(RUMServerHandle::receive_helper, args);
            rumtk_resolve_task!(&self.runtime, rumtk_spawn_task!(&self.runtime, task))
        }

        ///
        /// Sync API method for obtaining the client list of the server.
        ///
        pub fn get_clients(&self) -> ClientList {
            let args = rumtk_create_task_args!((Arc::clone(&self.server)));
            let task = rumtk_create_task!(RUMServerHandle::get_clients_helper, args);
            rumtk_resolve_task!(&self.runtime, rumtk_spawn_task!(&self.runtime, task))
        }

        ///
        /// Get the Address:Port info for this socket.
        ///
        pub fn get_address_info(&self) -> Option<RUMString> {
            let args = rumtk_create_task_args!(Arc::clone(&self.server));
            let task = rumtk_create_task!(RUMServerHandle::get_address_helper, args);
            rumtk_resolve_task!(&self.runtime, rumtk_spawn_task!(&self.runtime, task))
        }

        async fn send_helper(args: &SafeTaskArgs<Self::SendArgs>) -> RUMResult<()> {
            let owned_args = Arc::clone(args).clone();
            let locked_args = owned_args.read().await;
            let (server_ref, client_id, msg) = locked_args.get(0).unwrap();
            let mut server = server_ref.write().await;
            Ok(server.push_message(client_id, msg.clone()).await)
        }

        async fn receive_helper(
            args: &SafeTaskArgs<Self::ReceiveArgs>,
        ) -> RUMResult<RUMNetMessage> {
            let owned_args = Arc::clone(args).clone();
            let locked_args = owned_args.read().await;
            let (server_ref, client_id) = locked_args.get(0).unwrap();
            let mut server = server_ref.write().await;
            let mut msg = server.pop_message(&client_id).await;
            std::mem::drop(server);

            while msg.is_none() {
                let mut server = server_ref.write().await;
                msg = server.pop_message(&client_id).await;
            }
            Ok(msg.unwrap())
        }

        async fn start_helper(args: &SafeTaskArgs<Self::SelfArgs>) -> RUMResult<()> {
            let owned_args = Arc::clone(args).clone();
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let server_ref = locked_args.get(0).unwrap();
            RUMServer::run(&server_ref).await
        }

        async fn stop_helper(args: &SafeTaskArgs<Self::SelfArgs>) -> RUMResult<RUMString> {
            let owned_args = Arc::clone(args).clone();
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let server_ref = locked_args.get(0).unwrap();
            RUMServer::stop_server(server_ref).await
        }

        async fn new_helper(args: &SafeTaskArgs<ConnectionInfo>) -> TaskResult<RUMServer> {
            let owned_args = Arc::clone(args);
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let (ip, port) = match locked_args.get(0) {
                Some((ip, port)) => (ip, port),
                None => {
                    return Err(format_compact!(
                        "No IP address or port provided for connection!"
                    ))
                }
            };
            Ok(vec![RUMServer::new(ip, *port).await?])
        }

        async fn get_clients_helper(args: &SafeTaskArgs<Self::SelfArgs>) -> ClientList {
            let owned_args = Arc::clone(args).clone();
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let server_ref = locked_args.get(0).unwrap();
            let server = server_ref.read().await;
            server.get_clients().await
        }

        async fn get_address_helper(args: &SafeTaskArgs<Self::SelfArgs>) -> Option<RUMString> {
            let owned_args = Arc::clone(args).clone();
            let locked_args = owned_args.read().await;
            let server_ref = locked_args.get(0).unwrap();
            let mut server = server_ref.read().await;
            server.get_address_info().await
        }
    }
}

///
/// This module provides the preferred API for interacting and simplifying work with the [tcp]
/// module's primitives.
///
/// The API here is defined in the form of macros!
///
pub mod tcp_macros {

    ///
    /// Macro for creating a server instance.
    ///
    /// If a `port` is passed, we return the default configured [tcp::RUMServerHandle] instance
    /// exposed to the world on all interfaces.
    ///
    /// If an `ip` and `port` is passed, we create an instance of [tcp::RUMServerHandle] bound
    /// to that ip/port combo using the default number of threads on the system which should match
    /// roughly to the number of cores/threads.
    ///
    /// Alternatively, you can pass the `ip`, `port`, and `threads`. In such a case, the constructed
    /// [tcp::RUMServerHandle] will use only the number of threads requested.
    ///
    #[macro_export]
    macro_rules! rumtk_create_server {
        ( $port:expr ) => {{
            use $crate::net::tcp::RUMServerHandle;
            RUMServerHandle::default($port)
        }};
        ( $ip:expr, $port:expr ) => {{
            use $crate::net::tcp::RUMServerHandle;
            use $crate::threading::threading_functions::get_default_system_thread_count;
            RUMServerHandle::new($ip, $port, get_default_system_thread_count())
        }};
        ( $ip:expr, $port:expr, $threads:expr ) => {{
            use $crate::net::tcp::RUMServerHandle;
            RUMServerHandle::new($ip, $port, $threads)
        }};
    }

    ///
    /// Macro for starting the server. When a server is created, it does not start accepting clients
    /// right away. You need to call this macro to do that or call [tcp::RUMServerHandle::start]
    /// directly.
    ///
    /// The only argument that we expect is the `blocking` argument. If `blocking` is requested,
    /// calling this macro will block the calling thread. By default, we start the server in
    /// non-blocking mode so that you can do other actions in the calling thread like queueing
    /// messages.
    ///
    #[macro_export]
    macro_rules! rumtk_start_server {
        ( $server:expr ) => {{
            $server.start(false)
        }};
        ( $server:expr, $blocking:expr ) => {{
            $server.start($blocking)
        }};
    }

    ///
    /// This macro is a convenience macro that allows you to establish a connection to an endpoint.
    /// It creates and instance of [tcp::RUMClientHandle].
    ///
    /// If you only pass the `port`, we will connect to a server in *localhost* listening at that
    /// port.
    ///
    /// If you pass both `ip` and `port`, we will connect to a server listening at that ip/port
    /// combo.
    ///
    #[macro_export]
    macro_rules! rumtk_connect {
        ( $port:expr ) => {{
            use $crate::net::tcp::{RUMClientHandle, LOCALHOST};
            RUMClientHandle::connect(LOCALHOST, $port)
        }};
        ( $ip:expr, $port:expr ) => {{
            use $crate::net::tcp::RUMClientHandle;
            RUMClientHandle::connect($ip, $port)
        }};
    }
}
