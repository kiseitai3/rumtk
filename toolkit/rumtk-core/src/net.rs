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
    use std::ffi::CStr;
    use std::future::Future;
    use std::ops::DerefMut;
    use std::task::Poll;
    use tokio::sync::{Mutex as AsyncMutex, RwLock as AsyncRwLock};
    use std::sync::{Arc, Mutex};
    use ahash::{HashMap, HashMapExt};
    use compact_str::{format_compact, ToCompactString};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use crate::core::RUMResult;
    use crate::strings::{RUMArrayConversions, RUMString, RUMStringConversions};
    pub use tokio::net::{TcpListener, TcpStream};
    use tokio::{io, runtime};
    use crate::queue::queue::{TaskQueue};
    use crate::{rumtk_async_sleep, rumtk_create_task, rumtk_create_task_args, rumtk_init_threads, rumtk_resolve_task, rumtk_sleep, rumtk_spawn_task, rumtk_wait_on_task};
    use crate::threading::thread_primitives::{SafeTaskArgs, SafeTokioRuntime, TaskItems, TaskResult};
    use crate::threading::threading_functions::get_default_system_thread_count;

    const MESSAGE_BUFFER_SIZE: usize = 1024;


    pub type RUMNetMessage = Vec<u8>;
    type RUMNetPartialMessage = (RUMNetMessage, bool);
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

        pub async fn send(&mut self, msg: &RUMNetMessage) -> RUMResult<()> {
            match self.socket.write_all(msg.as_slice()).await {
                Ok(_) => Ok(()),
                Err(e) => Err(format_compact!("Unable to send message to {} because {}", &self.socket.local_addr().unwrap().to_compact_string(), &e)),
            }
        }

        pub async fn recv(&mut self) -> RUMResult<RUMNetMessage> {
            let mut msg = RUMNetMessage::new();
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
                Ok(n) => Ok((RUMNetMessage::from(buf), true)),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok((RUMNetMessage::from(buf), false)),
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

    pub type ClientList = Vec<RUMClient>;
    type SafeQueue<T> = Arc<AsyncMutex<VecDeque<T>>>;
    type SafeClients = Arc<AsyncMutex<ClientList>>;
    type SafeMappedQueues = Arc<AsyncMutex<HashMap<RUMString, SafeQueue<RUMNetMessage>>>>;
    pub type SafeListener = Arc<AsyncMutex<TcpListener>>;
    pub type SafeServer = Arc<AsyncRwLock<RUMServer>>;
    pub type SafeClient = Arc<AsyncMutex<RUMClient>>;

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
        tx_in: SafeQueue<RUMNetMessage>,
        tx_out: SafeMappedQueues,
        clients: SafeClients,
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
                Err(e) => return Err(format_compact!("Unable to bind to {} because {}", &addr.as_str(), &e)),
            };
            let tx_in = SafeQueue::<RUMNetMessage>::new(AsyncMutex::new(VecDeque::new()));
            let tx_out = SafeMappedQueues::new(AsyncMutex::new(HashMap::<RUMString, SafeQueue<RUMNetMessage>>::new()));
            let clients = SafeClients::new(AsyncMutex::new(Vec::new()));
            let tcp_listener = Arc::new(AsyncMutex::new(tcp_listener_handle));
            Ok(RUMServer{tcp_listener, tx_in, tx_out, clients, stop: false, shutdown_completed: false})
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
            let mut accept_handle = tokio::spawn(RUMServer::handle_accept(Arc::clone(&reowned_self.tcp_listener), Arc::clone(&reowned_self.clients)));
            let mut send_handle = tokio::spawn(RUMServer::handle_send(Arc::clone(&reowned_self.clients), Arc::clone(&reowned_self.tx_out)));
            let mut receive_handle = tokio::spawn(RUMServer::handle_receive(Arc::clone(&reowned_self.clients), Arc::clone(&reowned_self.tx_in)));
            let mut stop = reowned_self.stop;
            //Most drop here to allow the outside world to grab access to the server handle and interact with us.
            std::mem::drop(reowned_self); //Bootstrap magic that let's the outside able to interact with our server while it runs autonomously in the background.
            // Essentially, repeat the above but inside a scope thus automatically freeing the handle to outside access on a routine basis.
            while !stop {
                let mut reowned_self = ctx.read().await;
                if accept_handle.is_finished() {
                    accept_handle = tokio::spawn(RUMServer::handle_accept(Arc::clone(&reowned_self.tcp_listener), Arc::clone(&reowned_self.clients)));
                }
                if send_handle.is_finished() {
                    send_handle = tokio::spawn(RUMServer::handle_send(Arc::clone(&reowned_self.clients), Arc::clone(&reowned_self.tx_out)));
                }
                if receive_handle.is_finished() {
                    receive_handle = tokio::spawn(RUMServer::handle_receive(Arc::clone(&reowned_self.clients),Arc::clone(&reowned_self.tx_in)));
                }
                stop = reowned_self.stop;
            }
            while !accept_handle.is_finished() || !send_handle.is_finished() || !receive_handle.is_finished() {
                rumtk_async_sleep!(0.001);
            }
            println!("Shutting down server!");
            Ok(())
        }

        ///
        /// This method signals the server to stop.
        ///
        async fn stop_server(&mut self) -> RUMResult<RUMString> {
            self.stop = true;
            while !self.shutdown_completed {}
            Ok(format_compact!("Server shutdown..."))
        }

        ///
        /// Contains basic logic for listening for incoming connections.
        ///
        async fn handle_accept(listener: SafeListener, clients: SafeClients) {
            let server = listener.lock().await;
            match server.accept().await {
                Ok((socket, _)) => {
                    let mut client_list = clients.lock().await;
                    let client = match RUMClient::accept(socket).await {
                        Ok(client) => client,
                        Err(e) => return (),
                    };
                    client_list.push(client);
                }
                Err(e) => ()
            }
        }

        ///
        /// Contains logic for sending messages queued for a client to it. `tx_out` is a reference
        /// of [SafeMappedQueues] which is a hash map of [SafeQueue<RUMNetMessage>] whose keys are
        /// the client's peer address string.
        ///
        async fn handle_send(clients: SafeClients, tx_out: SafeMappedQueues) {
            let mut client_list = clients.lock().await;
            for mut client in client_list.iter_mut() {
                let ready = client.write_ready().await;
                if ready {
                    let addr = client.socket.peer_addr().unwrap().to_compact_string();
                    let mut queues = tx_out.lock().await;
                    let mut queue = match queues.get_mut(&addr) {
                        Some(queue) => queue,
                        None => continue,
                    };
                    let mut locked_queue = queue.lock().await;
                    for msg in locked_queue.iter() {
                        match client.send(&msg).await {
                            Ok(_) => (),
                            Err(e) => return (),
                        }
                    }
                    locked_queue.clear();
                }
            }
        }

        ///
        /// Contains the logic for handling receiving messages from clients. Incoming messages are
        /// all placed into a queue that the "outside" world can interact with.
        ///
        async fn handle_receive(clients: SafeClients, tx_in: SafeQueue<RUMNetMessage>) {
            let mut client_list = clients.lock().await;
            for mut client in client_list.iter_mut() {
                let ready = client.read_ready().await;
                if ready {
                    let msg = client.recv().await.unwrap();
                    let mut queue = tx_in.lock().await;
                    queue.push_back(msg);
                }
            }
        }

        ///
        /// Return list of clients.
        ///
        pub async fn get_clients(&self) -> ClientList {
            let clients = self.clients.lock().await;
            ClientList::from(clients.iter())
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
        pub async fn pop_message(&mut self) -> Option<RUMNetMessage> {
            let mut queue = self.tx_in.lock().await;
            queue.pop_front()
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
        type ReceiveArgs<> = SafeClient;

        pub fn connect(ip: &str, port: u16) -> RUMResult<RUMClientHandle> {
            RUMClientHandle::new(ip, port)
        }

        pub fn new(ip: &str, port: u16) -> RUMResult<RUMClientHandle> {
            let runtime = rumtk_init_threads!(&1);
            let con: ConnectionInfo = (RUMString::from(ip), port);
            let args = rumtk_create_task_args!(con);
            let client = rumtk_wait_on_task!(&runtime, RUMClientHandle::new_helper, &args)?.pop().unwrap();
            Ok(RUMClientHandle{client: SafeClient::new(AsyncMutex::new(client)), runtime})
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

        async fn send_helper(args: &SafeTaskArgs<Self::SendArgs<'_>>) -> RUMResult<()> {
            let owned_args = Arc::clone(args).clone();
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let (client_lock_ref, msg) = locked_args.get(0).unwrap();
            let mut client_ref = Arc::clone(client_lock_ref);
            let mut client = client_ref.lock().await;
            client.send(msg).await
        }

        async fn receive_helper(args: &SafeTaskArgs<Self::ReceiveArgs>) -> RUMResult<RUMNetMessage> {
            let owned_args = Arc::clone(args).clone();
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let mut client_ref = locked_args.get(0).unwrap();
            let mut client = client_ref.lock().await;
            client.recv().await
        }

        async fn new_helper(args: &SafeTaskArgs<ConnectionInfo>) -> TaskResult<RUMClient> {
            let owned_args = Arc::clone(args);
            let lock_future = owned_args.read().await;
            let (ip, port) = match lock_future.get(0) {
                Some((ip, port)) => (ip, port),
                None => return Err(format_compact!("No IP address or port provided for connection!")),
            };
            Ok(vec![RUMClient::connect(ip, *port).await?])
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
        type ReceiveArgs = SafeServer;

        ///
        /// Constructs a [RUMServerHandle] using the detected number of parallel units/threads on
        /// this machine. This method automatically binds to IP 0.0.0.0. Meaning, your server may
        /// become visible to the outside world.
        ///
        pub fn default(port: u16) -> RUMResult<RUMServerHandle> {
            RUMServerHandle::new("0.0.0.0", port, get_default_system_thread_count())
        }

        ///
        /// Constructs a [RUMServerHandle] using the detected number of parallel units/threads on
        /// this machine. This method automatically binds to **localhost**. Meaning, your server
        /// remains private in your machine.
        ///
        pub fn default_local(port: u16) -> RUMResult<RUMServerHandle> {
            RUMServerHandle::new("localhost", port, get_default_system_thread_count())
        }

        ///
        /// General purpose constructor for [RUMServerHandle]. It takes an ip and port and binds it.
        /// You can also control how many threads are spawned under the hood for this server handle.
        ///
        pub fn new(ip: &str, port: u16, threads: usize) -> RUMResult<RUMServerHandle> {
            let runtime = rumtk_init_threads!(&threads);
            let con: ConnectionInfo = (RUMString::from(ip), port);
            let args = rumtk_create_task_args!(con);
            let server = rumtk_wait_on_task!(&runtime, RUMServerHandle::new_helper, &args)?.pop().unwrap();
            Ok(RUMServerHandle{server: Arc::new(AsyncRwLock::new(server)), runtime})
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
        pub fn send(&mut self, client_id: &RUMString, msg: RUMNetMessage) -> RUMResult<()> {
            let args = rumtk_create_task_args!((Arc::clone(&mut self.server), client_id.clone(), msg));
            let task = rumtk_create_task!(RUMServerHandle::send_helper, args);
            rumtk_resolve_task!(&self.runtime, rumtk_spawn_task!(&self.runtime, task))
        }

        ///
        /// Sync API method for obtaining a single message from the server's incoming queue.
        ///
        pub fn receive(&mut self) -> RUMNetMessage {
            let args = rumtk_create_task_args!((Arc::clone(&mut self.server)));
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

        async fn send_helper(args: &SafeTaskArgs<Self::SendArgs>) -> RUMResult<()> {
            let owned_args = Arc::clone(args).clone();
            let locked_args = owned_args.read().await;
            let (server_ref, client_id, msg) = locked_args.get(0).unwrap();
            let mut server = server_ref.write().await;
            Ok(server.push_message(client_id, msg.to_vec()).await)
        }

        async fn receive_helper(args: &SafeTaskArgs<Self::ReceiveArgs>) -> RUMNetMessage {
            let owned_args = Arc::clone(args).clone();
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let mut server_ref = locked_args.get(0).unwrap();
            let mut server = server_ref.write().await;
            let mut msg = server.pop_message().await;
            while msg.is_none() {
                msg = server.pop_message().await;
            }
            msg.unwrap()
        }

        async fn start_helper(args: &SafeTaskArgs<Self::ReceiveArgs>) -> RUMResult<()> {
            let owned_args = Arc::clone(args).clone();
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let server_ref = locked_args.get(0).unwrap();
            RUMServer::run(&server_ref).await
        }

        async fn stop_helper(args: &SafeTaskArgs<Self::ReceiveArgs>) -> RUMResult<RUMString> {
            let owned_args = Arc::clone(args).clone();
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let mut server_ref = locked_args.get(0).unwrap();
            let mut server = server_ref.write().await;
            server.stop_server().await
        }

        async fn new_helper(args: &SafeTaskArgs<ConnectionInfo>) -> TaskResult<RUMServer> {
            let owned_args = Arc::clone(args);
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let (ip, port) = match locked_args.get(0) {
                Some((ip, port)) => (ip, port),
                None => return Err(format_compact!("No IP address or port provided for connection!")),
            };
            Ok(vec![RUMServer::new(ip, *port).await?])
        }

        async fn get_clients_helper(args: &SafeTaskArgs<Self::ReceiveArgs>) -> ClientList {
            let owned_args = Arc::clone(args).clone();
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let server_ref = locked_args.get(0).unwrap();
            let server = server_ref.write().await;
            server.get_clients()
        }
    }
}

pub mod tcp_macros {

    #[macro_export]
    macro_rules! rumtk_create_server {
        ( $port:expr ) => {{
            use $crate::net::tcp::{RUMServerHandle};
            RUMServerHandle::default($port)
        }};
        ( $ip:expr, $port:expr ) => {{
            use crate::threading::threading_functions::get_default_system_thread_count;
            use $crate::net::tcp::{RUMServerHandle};
            RUMServerHandle::new($ip, $port, get_default_system_thread_count())
        }};
        ( $ip:expr, $port:expr, $threads:expr ) => {{
            use $crate::net::tcp::{RUMServerHandle};
            RUMServerHandle::new($ip, $port, $threads)
        }};
    }

    #[macro_export]
    macro_rules! rumtk_start_server {
        ( $server:expr ) => {{
            $server.start(false)
        }};
        ( $server:expr, $blocking:expr ) => {{
            $server.start($blocking)
        }};
    }

    #[macro_export]
    macro_rules! rumtk_connect {
        ( $port:expr ) => {{
            use $crate::net::tcp::{RUMClientHandle};
            RUMClientHandle::connect("localhost", $port)
        }};
        ( $ip:expr, $port:expr ) => {{
            use $crate::net::tcp::{RUMClientHandle};
            RUMClientHandle::connect($ip, $port)
        }};
    }
}

