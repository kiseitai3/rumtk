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
    use crate::strings::RUMString;
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
    pub use tokio::sync::{
        Mutex as AsyncMutex, MutexGuard as AsyncMutexGuard, RwLock as AsyncRwLock, RwLockReadGuard,
        RwLockWriteGuard,
    };

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
        disconnected: bool,
    }

    impl RUMClient {
        ///
        /// Connect to peer and construct the client.
        ///
        pub async fn connect(ip: &str, port: u16) -> RUMResult<RUMClient> {
            let addr = format_compact!("{}:{}", ip, port);
            match TcpStream::connect(addr.as_str()).await {
                Ok(socket) => Ok(RUMClient {
                    socket,
                    disconnected: false,
                }),
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
            Ok(RUMClient {
                socket,
                disconnected: false,
            })
        }

        ///
        /// Send message to server.
        ///
        pub async fn send(&mut self, msg: &RUMNetMessage) -> RUMResult<()> {
            if self.is_disconnected() {
                return Err(format_compact!(
                    "{} disconnected!",
                    &self.socket.peer_addr().unwrap().to_compact_string()
                ));
            }

            match self.socket.write_all(msg.as_slice()).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    self.disconnect();
                    Err(format_compact!(
                        "Unable to send message to {} because {}",
                        &self.socket.local_addr().unwrap().to_compact_string(),
                        &e
                    ))
                }
            }
        }

        ///
        /// Receive message from server. This method will make calls to [RUMClient::recv_some]
        /// indefinitely until we have the full message or stop receiving any data.
        ///
        pub async fn recv(&mut self) -> RUMResult<RUMNetMessage> {
            let mut msg = RUMNetMessage::new();

            if self.is_disconnected() {
                return Err(format_compact!(
                    "{} disconnected!",
                    &self.socket.peer_addr().unwrap().to_compact_string()
                ));
            }

            loop {
                let mut fragment = self.recv_some().await?;
                msg.append(&mut fragment.0);
                if fragment.1 == false {
                    break;
                }
            }
            Ok(msg)
        }

        async fn recv_some(&mut self) -> RUMResult<RUMNetPartialMessage> {
            let mut buf: [u8; MESSAGE_BUFFER_SIZE] = [0; MESSAGE_BUFFER_SIZE];
            match self.socket.try_read(&mut buf) {
                Ok(n) => match n {
                    0 => {
                        self.disconnect();
                        Err(format_compact!(
                            "Received 0 bytes from {}! It might have disconnected!",
                            &self.socket.peer_addr().unwrap().to_compact_string()
                        ))
                    }
                    MESSAGE_BUFFER_SIZE => Ok((RUMNetMessage::from(buf), true)),
                    _ => Ok((RUMNetMessage::from(buf[0..n].to_vec()), false)),
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    Ok((RUMNetMessage::new(), false))
                }
                Err(e) => {
                    self.disconnect();
                    Err(format_compact!(
                        "Error receiving message from {} because {}",
                        &self.socket.peer_addr().unwrap().to_compact_string(),
                        &e
                    ))
                }
            }
        }

        pub async fn wait_incoming(&self) -> RUMResult<bool> {
            let mut buf: [u8; 1] = [0; 1];

            if self.is_disconnected() {
                return Err(format_compact!(
                    "{} disconnected!",
                    &self.socket.peer_addr().unwrap().to_compact_string()
                ));
            }

            match self.socket.peek(&mut buf).await {
                Ok(n) => match n {
                    0 => Err(format_compact!(
                        "Received 0 bytes from {}! It might have disconnected!",
                        &self.socket.peer_addr().unwrap().to_compact_string()
                    )),
                    _ => Ok(true),
                },
                Err(e) => Err(format_compact!(
                    "Error receiving message from {} because {}. It might have disconnected!",
                    &self.socket.peer_addr().unwrap().to_compact_string(),
                    &e
                )),
            }
        }

        /// Check if socket is ready for reading.
        pub async fn read_ready(&self) -> bool {
            if self.is_disconnected() {
                return false;
            }

            match self.socket.readable().await {
                Ok(_) => true,
                Err(_) => false,
            }
        }

        /// Check if socket is ready for writing.
        pub async fn write_ready(&self) -> bool {
            if self.is_disconnected() {
                return false;
            }

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

        pub fn is_disconnected(&self) -> bool {
            self.disconnected
        }

        pub fn disconnect(&mut self) {
            self.disconnected = true;
        }
    }

    /// List of clients that you can interact with.
    pub type ClientList = Vec<SafeClient>;
    /// List of client IDs that you can interact with.
    pub type ClientIDList = Vec<RUMString>;
    type SafeQueue<T> = Arc<AsyncMutex<VecDeque<T>>>;
    pub type SafeClient = Arc<AsyncRwLock<RUMClient>>;
    type SafeClients = Arc<AsyncRwLock<HashMap<RUMString, SafeClient>>>;
    type SafeClientIDList = Arc<AsyncMutex<ClientIDList>>;
    type SafeMappedQueues = Arc<AsyncMutex<HashMap<RUMString, SafeQueue<RUMNetMessage>>>>;
    pub type SafeListener = Arc<AsyncMutex<TcpListener>>;
    pub type SafeServer = Arc<AsyncRwLock<RUMServer>>;

    async fn lock_client_ex(client: &SafeClient) -> RwLockWriteGuard<RUMClient> {
        let locked = client.write().await;
        locked
    }

    async fn lock_client(client: &SafeClient) -> RwLockReadGuard<RUMClient> {
        let locked = client.read().await;
        locked
    }

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
        pub async fn run(ctx: SafeServer) -> RUMResult<()> {
            // Bootstrapping the main server loop.
            let reowned_self = ctx.read().await;
            let mut accept_handle = tokio::spawn(RUMServer::handle_accept(
                Arc::clone(&reowned_self.tcp_listener),
                Arc::clone(&reowned_self.clients),
                Arc::clone(&reowned_self.tx_in),
                Arc::clone(&reowned_self.tx_out),
            ));
            let mut send_handle = tokio::spawn(RUMServer::handle_send(
                Arc::clone(&reowned_self.clients),
                Arc::clone(&reowned_self.tx_out),
            ));
            let mut receive_handle = tokio::spawn(RUMServer::handle_receive(
                Arc::clone(&reowned_self.clients),
                Arc::clone(&reowned_self.tx_in),
            ));
            let mut gc_handle = tokio::spawn(RUMServer::handle_client_gc(
                Arc::clone(&reowned_self.clients),
                Arc::clone(&reowned_self.tx_in),
                Arc::clone(&reowned_self.tx_out),
            ));
            let mut stop = reowned_self.stop;
            //Most drop here to allow the outside world to grab access to the server handle and interact with us.
            std::mem::drop(reowned_self); //Bootstrap magic that let's the outside able to interact with our server while it runs autonomously in the background.
                                          // Essentially, repeat the above but inside a scope thus automatically freeing the handle to outside access on a routine basis.
            while !stop {
                let reowned_self = ctx.read().await;
                if accept_handle.is_finished() {
                    accept_handle = tokio::spawn(RUMServer::handle_accept(
                        Arc::clone(&reowned_self.tcp_listener),
                        Arc::clone(&reowned_self.clients),
                        Arc::clone(&reowned_self.tx_in),
                        Arc::clone(&reowned_self.tx_out),
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
                if gc_handle.is_finished() {
                    gc_handle = tokio::spawn(RUMServer::handle_client_gc(
                        Arc::clone(&reowned_self.clients),
                        Arc::clone(&reowned_self.tx_in),
                        Arc::clone(&reowned_self.tx_out),
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
        pub async fn stop_server(ctx: &SafeServer) -> RUMResult<RUMString> {
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
        pub async fn handle_accept(
            listener: SafeListener,
            clients: SafeClients,
            tx_in: SafeMappedQueues,
            tx_out: SafeMappedQueues,
        ) -> RUMResult<()> {
            let server = listener.lock().await;
            match server.accept().await {
                Ok((socket, _)) => {
                    let client = RUMClient::accept(socket).await?;
                    let client_id = match client.get_address(false).await {
                        Some(client_id) => client_id,
                        None => return Err(format_compact!("Accepted client returned no peer address. This should not be happening!"))
                    };
                    let mut client_list = clients.write().await;
                    RUMServer::register_queue(&tx_in, &client_id).await;
                    RUMServer::register_queue(&tx_out, &client_id).await;
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
        pub async fn handle_send(clients: SafeClients, tx_out: SafeMappedQueues) -> RUMResult<()> {
            let mut client_list = clients.write().await;
            for (client_id, client) in client_list.iter_mut() {
                let messages = match RUMServer::pop_queue(&tx_out, client_id).await {
                    Some(messages) => messages,
                    None => continue,
                };
                for msg in messages.iter() {
                    match RUMServer::send(client, msg).await {
                        Ok(_) => (),
                        Err(e) => {
                            return Err(format_compact!("{}... Dropping client...", e));
                        }
                    };
                }
            }

            if client_list.is_empty() {
                rumtk_async_sleep!(0.1).await;
            }
            Ok(())
        }

        ///
        /// Contains the logic for handling receiving messages from clients. Incoming messages are
        /// all placed into a queue that the "outside" world can interact with.
        ///
        pub async fn handle_receive(
            clients: SafeClients,
            tx_in: SafeMappedQueues,
        ) -> RUMResult<()> {
            let mut client_list = clients.write().await;
            for (client_id, client) in client_list.iter_mut() {
                let msg = RUMServer::receive(client).await?;
                if !msg.is_empty() {
                    RUMServer::push_queue(&tx_in, client_id, msg).await?;
                }
            }
            if client_list.is_empty() {
                rumtk_async_sleep!(0.1).await;
            }
            Ok(())
        }

        ///
        /// Contains the logic for handling removal of clients from the server if they disconnected.
        ///
        pub async fn handle_client_gc(
            clients: SafeClients,
            tx_in: SafeMappedQueues,
            tx_out: SafeMappedQueues,
        ) -> RUMResult<()> {
            let mut client_list = clients.write().await;
            let client_keys = client_list.keys().cloned().collect::<Vec<_>>();
            let mut disconnected_clients = Vec::<RUMString>::with_capacity(client_list.len());
            for client_id in client_keys {
                let disconnected = client_list[&client_id].write().await.is_disconnected();
                let empty_queues = RUMServer::is_queue_empty(&tx_in, &client_id).await
                    && RUMServer::is_queue_empty(&tx_out, &client_id).await;
                if disconnected && empty_queues {
                    client_list.remove(&client_id);
                    tx_in.lock().await.remove(&client_id);
                    tx_out.lock().await.remove(&client_id);
                    disconnected_clients.push(client_id);
                }
            }

            if !disconnected_clients.is_empty() {
                return Err(format_compact!(
                    "The following clients have disconnected and thus will be removed! {:?}",
                    disconnected_clients
                ));
            }

            Ok(())
        }

        pub async fn register_queue(tx_queues: &SafeMappedQueues, client: &RUMString) {
            let mut queues = tx_queues.lock().await;
            let new_queue = SafeQueue::<RUMNetMessage>::new(AsyncMutex::new(VecDeque::new()));
            queues.insert(client.clone(), new_queue);
        }

        pub async fn push_queue(
            tx_queues: &SafeMappedQueues,
            client: &RUMString,
            msg: RUMNetMessage,
        ) -> RUMResult<()> {
            let mut queues = tx_queues.lock().await;
            let mut queue = match queues.get_mut(client) {
                Some(queue) => queue,
                None => {
                    return Err(format_compact!("Attempted to queue message for non-connected \
                    client! Make sure client was connected! The client might have been disconnected. \
                    Client: {}", &client));
                }
            };
            let mut locked_queue = queue.lock().await;
            locked_queue.push_back(msg);
            Ok(())
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

        pub async fn is_queue_empty(tx_queues: &SafeMappedQueues, client: &RUMString) -> bool {
            let queues = tx_queues.lock().await;
            let queue = match queues.get(client) {
                Some(queue) => queue,
                None => return true,
            };
            let empty = queue.lock().await.is_empty();
            empty
        }

        pub async fn send(client: &SafeClient, msg: &RUMNetMessage) -> RUMResult<()> {
            let mut owned_client = lock_client_ex(client).await;
            owned_client.send(msg).await
        }

        pub async fn receive(client: &SafeClient) -> RUMResult<RUMNetMessage> {
            let mut owned_client = lock_client_ex(client).await;
            owned_client.recv().await
        }

        pub async fn disconnect(client: &SafeClient) {
            let mut owned_client = lock_client_ex(client).await;
            owned_client.disconnect()
        }

        pub async fn get_client(
            clients: &SafeClients,
            client: &RUMString,
        ) -> RUMResult<SafeClient> {
            match clients.read().await.get(client) {
                Some(client) => Ok(client.clone()),
                _ => Err(format_compact!("Client {} not found!", client)),
            }
        }

        ///
        /// Return client id list.
        ///
        pub async fn get_client_ids(clients: &SafeClients) -> ClientIDList {
            clients.read().await.keys().cloned().collect::<Vec<_>>()
        }

        pub async fn get_client_id(client: &SafeClient) -> RUMString {
            lock_client(client)
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
                SOCKET_READINESS_TYPE::READ_READY => lock_client(client).await.read_ready().await,
                SOCKET_READINESS_TYPE::WRITE_READY => lock_client(client).await.write_ready().await,
                SOCKET_READINESS_TYPE::READWRITE_READY => {
                    let locked_client = lock_client(client).await;
                    locked_client.read_ready().await && locked_client.write_ready().await
                }
            }
        }

        ///
        /// Return list of clients.
        ///
        pub async fn get_clients(&self) -> ClientList {
            let owned_clients = self.clients.read().await;
            let mut clients = ClientList::with_capacity(owned_clients.len());
            for (client_id, client) in owned_clients.iter() {
                clients.push(client.clone());
            }
            clients
        }

        ///
        /// Queues a message onto the server to send to client.
        ///
        pub async fn push_message(
            &mut self,
            client_id: &RUMString,
            msg: RUMNetMessage,
        ) -> RUMResult<()> {
            let mut queue = self.tx_out.lock().await;
            if !queue.contains_key(client_id) {
                return Err(format_compact!("No client with id {} found!", &client_id));
            }
            let mut queue = queue[client_id].lock().await;
            queue.push_back(msg);
            Ok(())
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
        /// Obtain a message, if available, from the incoming queue.
        ///
        pub async fn wait_incoming(&mut self, client_id: &RUMString) -> RUMResult<bool> {
            let client = RUMServer::get_client(&self.clients, client_id).await?;
            let owned_client = client.write().await;
            owned_client.wait_incoming().await
        }

        ///
        /// Get the Address:Port info for this socket.
        ///
        pub async fn get_address_info(&self) -> Option<RUMString> {
            self.address.clone()
        }

        ///
        /// Attempts to clear clients that have been marked as disconnected.
        ///
        pub async fn gc_clients(&mut self) -> RUMResult<()> {
            RUMServer::handle_client_gc(
                self.clients.clone(),
                self.tx_in.clone(),
                self.tx_out.clone(),
            )
            .await
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
            rumtk_resolve_task!(&self.runtime, rumtk_spawn_task!(&self.runtime, task))?
        }

        ///
        /// Sync API method for obtaining a single message from the server's incoming queue.
        /// Returns the next available [RUMNetMessage]
        ///
        pub fn receive(&mut self, client_id: &RUMString) -> RUMResult<RUMNetMessage> {
            let args = rumtk_create_task_args!((Arc::clone(&mut self.server), client_id.clone()));
            let task = rumtk_create_task!(RUMServerHandle::receive_helper, args);
            rumtk_resolve_task!(&self.runtime, rumtk_spawn_task!(&self.runtime, task))?
        }

        ///
        /// Sync API method for obtaining the client list of the server.
        ///
        pub fn get_clients(&self) -> ClientList {
            let args = rumtk_create_task_args!((Arc::clone(&self.server)));
            let task = rumtk_create_task!(RUMServerHandle::get_clients_helper, args);
            rumtk_resolve_task!(&self.runtime, rumtk_spawn_task!(&self.runtime, task)).unwrap()
        }

        ///
        /// Sync API method for obtaining the client list of the server.
        ///
        pub fn get_client_ids(&self) -> ClientIDList {
            let args = rumtk_create_task_args!((Arc::clone(&self.server)));
            let task = rumtk_create_task!(RUMServerHandle::get_client_ids_helper, args);
            rumtk_resolve_task!(&self.runtime, rumtk_spawn_task!(&self.runtime, task)).unwrap()
        }

        ///
        /// Garbage Collection API method for dropping clients flagged as disconnected.
        ///
        pub fn gc_clients(&self) -> RUMResult<()> {
            let args = rumtk_create_task_args!((Arc::clone(&self.server)));
            let task = rumtk_create_task!(RUMServerHandle::gc_clients_helper, args);
            rumtk_resolve_task!(&self.runtime, rumtk_spawn_task!(&self.runtime, task))?
        }

        ///
        /// Get the Address:Port info for this socket.
        ///
        pub fn get_address_info(&self) -> Option<RUMString> {
            let args = rumtk_create_task_args!(Arc::clone(&self.server));
            let task = rumtk_create_task!(RUMServerHandle::get_address_helper, args);
            rumtk_resolve_task!(&self.runtime, rumtk_spawn_task!(&self.runtime, task))
                .expect("Expected an address:port for this client.")
        }

        async fn send_helper(args: &SafeTaskArgs<Self::SendArgs>) -> RUMResult<()> {
            let owned_args = Arc::clone(args).clone();
            let locked_args = owned_args.read().await;
            let (server_ref, client_id, msg) = locked_args.get(0).unwrap();
            let mut server = server_ref.write().await;
            Ok(server.push_message(client_id, msg.clone()).await?)
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
            RUMServer::run(server_ref.clone()).await
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

        async fn get_client_ids_helper(args: &SafeTaskArgs<Self::SelfArgs>) -> ClientIDList {
            let owned_args = Arc::clone(args).clone();
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let server_ref = locked_args.get(0).unwrap();
            let server = server_ref.read().await;
            RUMServer::get_client_ids(&server.clients).await
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

        async fn gc_clients_helper(args: &SafeTaskArgs<Self::SelfArgs>) -> RUMResult<()> {
            let owned_args = Arc::clone(args).clone();
            let locked_args = owned_args.read().await;
            let server_ref = locked_args.get(0).unwrap();
            let mut server = server_ref.write().await;
            server.gc_clients().await
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

    ///
    /// Convenience macro for obtaining the ip and port off a string with format `ip:port`.
    ///
    /// # Example Usage
    ///
    /// ```
    /// use rumtk_core::{rumtk_create_server, rumtk_get_ip_port};
    ///
    /// let server = rumtk_create_server!(0).unwrap();
    /// let ip_addr_info = server.get_address_info().unwrap();
    /// let (ip, port) = rumtk_get_ip_port!(&ip_addr_info);
    /// assert!(port > 0, "Expected non-zero port!");
    /// ```
    ///
    #[macro_export]
    macro_rules! rumtk_get_ip_port {
        ( $address_str:expr ) => {{
            use $crate::strings::RUMStringConversions;
            let mut components = $address_str.split(':');
            (
                components.next().unwrap().to_rumstring(),
                components.next().unwrap().parse::<u16>().unwrap(),
            )
        }};
    }
}
