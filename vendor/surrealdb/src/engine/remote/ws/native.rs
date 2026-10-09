use std::collections::HashSet;
use std::sync::Arc;

use async_channel::Receiver;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use surrealdb_types::{ConnectionError, ValidationError};
use tokio::net::TcpStream;
use tokio::sync::{RwLock, watch};
use tokio::time;
use tokio::time::MissedTickBehavior;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::error::Error as WsError;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::http::header::SEC_WEBSOCKET_PROTOCOL;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream};
use uuid::Uuid;

use super::{
    HandleResult, PATH, PING_INTERVAL, SessionState, WsMessage, create_ping_message,
    handle_response, handle_route, handle_session, replay_session, reset_sessions,
};
use crate::conn::{self, Route, Router};
use crate::engine::{IntervalStream, SessionError};
use crate::method::BoxFuture;
#[cfg(any(feature = "native-tls", feature = "rustls"))]
use crate::opt::Tls;
use crate::opt::{Endpoint, WaitFor};
use crate::types::HashMap;
use crate::{Error, ExtraFeatures, SessionClone, SessionId, Surreal};

pub(crate) const NAGLE_ALG: bool = false;

type MessageSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type MessageStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type Sessions = HashMap<Uuid, Result<Arc<SessionState>, SessionError>>;

// ============================================================================
// Platform Implementation
// ============================================================================

impl WsMessage for Message {
    fn binary(payload: Vec<u8>) -> Self {
        Message::Binary(payload.into())
    }

    fn as_binary(&self) -> Option<&[u8]> {
        match self {
            Message::Binary(data) => Some(data),
            _ => None,
        }
    }

    fn should_process(&self) -> bool {
        matches!(self, Message::Binary(_))
    }

    fn log_description(&self) -> &'static str {
        match self {
            Message::Text(_) => "text message",
            Message::Binary(_) => "binary message",
            Message::Ping(_) => "ping",
            Message::Pong(_) => "pong",
            Message::Frame(_) => "raw frame",
            Message::Close(_) => "close message",
        }
    }
}

#[cfg(any(feature = "native-tls", feature = "rustls"))]
impl From<Tls> for Connector {
    fn from(tls: Tls) -> Self {
        match tls {
            #[cfg(feature = "native-tls")]
            Tls::Native(config) => Self::NativeTls(config),
            #[cfg(feature = "rustls")]
            Tls::Rust(config) => Self::Rustls(std::sync::Arc::new(config)),
        }
    }
}

pub(crate) async fn connect(
    endpoint: &Endpoint,
    config: Option<WebSocketConfig>,
    #[cfg_attr(
        not(any(feature = "native-tls", feature = "rustls")),
        expect(unused_variables)
    )]
    maybe_connector: Option<Connector>,
) -> crate::Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let mut request = (&endpoint.url).into_client_request().map_err(|err| {
        Error::validation(
            format!("Invalid URL: {}", err),
            ValidationError::InvalidRequest,
        )
    })?;

    request.headers_mut().insert(
        SEC_WEBSOCKET_PROTOCOL,
        HeaderValue::from_static("flatbuffers"),
    );

    #[cfg(any(feature = "native-tls", feature = "rustls"))]
    let (socket, _) = tokio_tungstenite::connect_async_tls_with_config(
        request,
        config,
        NAGLE_ALG,
        maybe_connector,
    )
    .await
    .map_err(|err| {
        Error::connection(
            format!("WebSocket error: {}", err),
            ConnectionError::ConnectionFailed,
        )
    })?;

    #[cfg(not(any(feature = "native-tls", feature = "rustls")))]
    let (socket, _) = tokio_tungstenite::connect_async_with_config(request, config, NAGLE_ALG)
        .await
        .map_err(|err| {
            Error::connection(
                format!("WebSocket error: {}", err),
                ConnectionError::ConnectionFailed,
            )
        })?;

    Ok(socket)
}

impl crate::Connection for super::Client {}
impl conn::Sealed for super::Client {
    #[allow(private_interfaces)]
    fn connect(
        mut address: Endpoint,
        capacity: usize,
        session_clone: Option<crate::SessionClone>,
    ) -> BoxFuture<'static, crate::Result<Surreal<Self>>> {
        Box::pin(async move {
            address.url = address
                .url
                .join(PATH)
                .map_err(|e| Error::validation(e.to_string(), ValidationError::InvalidRequest))?;
            #[cfg(any(feature = "native-tls", feature = "rustls"))]
            let maybe_connector = address.config.tls_config.clone().map(Connector::from);
            #[cfg(not(any(feature = "native-tls", feature = "rustls")))]
            let maybe_connector = None;

            let ws_config = WebSocketConfig::default()
                .read_buffer_size(address.config.websocket.read_buffer_size)
                .max_message_size(address.config.websocket.max_message_size)
                .max_frame_size(address.config.websocket.max_message_size)
                .max_write_buffer_size(address.config.websocket.max_write_buffer_size)
                .write_buffer_size(address.config.websocket.write_buffer_size);

            let socket = connect(&address, Some(ws_config), maybe_connector.clone()).await?;

            let bounded = address.config.bounded_requests;
            let (shutdown_tx, shutdown_rx) = async_channel::bounded(1);
            let capacity = if bounded { 34 } else { capacity };
            let (route_tx, route_rx) = match capacity {
                0 => async_channel::unbounded(),
                capacity => async_channel::bounded(capacity),
            };
            let config = address.config.clone();
            let session_clone = session_clone.unwrap_or_else(SessionClone::new);

            tokio::spawn(run_router(
                address,
                maybe_connector,
                ws_config,
                socket,
                route_rx,
                session_clone.receiver.clone(),
                Some(shutdown_tx),
            ));

            let mut features = HashSet::new();
            features.insert(ExtraFeatures::LiveQueries);

            let waiter = watch::channel(Some(WaitFor::Connection));
            let router = if bounded {
                Router::from_bounded_route_sender(route_tx, shutdown_rx, features, config)
            } else {
                Router::from_route_sender(route_tx, features, config)
            };

            Ok((router, waiter, session_clone).into())
        })
    }
}

// ============================================================================
// Router State
// ============================================================================

/// Router state for native WebSocket connections.
struct RouterState {
    sessions: Sessions,
    sink: RwLock<MessageSink>,
    stream: RwLock<MessageStream>,
}

impl RouterState {
    fn new(sink: MessageSink, stream: MessageStream) -> Self {
        RouterState {
            sessions: HashMap::new(),
            sink: RwLock::new(sink),
            stream: RwLock::new(stream),
        }
    }

    async fn update_connection(&self, sink: MessageSink, stream: MessageStream) {
        *self.sink.write().await = sink;
        *self.stream.write().await = stream;
    }
}

// ============================================================================
// Router
// ============================================================================

async fn router_reconnect(
    maybe_connector: &Option<Connector>,
    config: &WebSocketConfig,
    state: &RouterState,
    endpoint: &Endpoint,
) {
    loop {
        trace!("Reconnecting...");
        match connect(endpoint, Some(*config), maybe_connector.clone()).await {
            Ok(s) => {
                let (new_sink, new_stream) = s.split();
                state.update_connection(new_sink, new_stream).await;
                // Replay state for ALL sessions
                for (session_id, session_result) in state.sessions.to_vec() {
                    if let Ok(session_state) = session_result {
                        replay_session::<Message, _, _>(session_id, &session_state, &state.sink)
                            .await
                            .ok();
                    }
                }
                trace!("Reconnected successfully");
                break;
            }
            Err(_) => {
                trace!("Native WebSocket reconnect failed");
                time::sleep(time::Duration::from_secs(1)).await;
            }
        }
    }
}

pub(crate) async fn run_router(
    endpoint: Endpoint,
    maybe_connector: Option<Connector>,
    config: WebSocketConfig,
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    route_rx: Receiver<Route>,
    session_rx: Receiver<SessionId>,
    shutdown: Option<async_channel::Sender<()>>,
) {
    if endpoint.config.bounded_requests {
        run_bounded_router(
            endpoint,
            maybe_connector,
            config,
            socket,
            route_rx,
            session_rx,
        )
        .await;
        if let Some(shutdown) = shutdown {
            shutdown.close();
        }
        return;
    }
    let ping: Message = create_ping_message();

    let (socket_sink, socket_stream) = socket.split();
    let state = Arc::new(RouterState::new(socket_sink, socket_stream));

    'router: loop {
        let mut interval = time::interval(PING_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        let mut pinger = IntervalStream::new(interval);

        reset_sessions(&state.sessions).await;

        loop {
            tokio::select! {
                biased;

                session = session_rx.recv() => {
                    let Ok(session_id) = session else {
                        break 'router
                    };
                    handle_session::<Message, _, _>(session_id, &state.sessions, &state.sink).await;
                }
                route = route_rx.recv() => {
                    let Ok(route) = route else {
                        match state.sink.write().await.send(Message::Close(None)).await {
                            Ok(..) => trace!("Connection closed successfully"),
                            Err(_) => warn!("Native WebSocket close failed")
                        }
                        break 'router;
                    };

                    // Apply any session-lifecycle events that were enqueued before this
                    // query. The session channel is separate from the route channel, so a
                    // freshly registered or cloned session may not have been processed yet;
                    // receiving this route establishes a happens-before with the sender, so
                    // those earlier events are now observable and one drain pass suffices.
                    while let Ok(session_id) = session_rx.try_recv() {
                        handle_session::<Message, _, _>(session_id, &state.sessions, &state.sink).await;
                    }

                    match handle_route::<Message, _, _>(
                        route, config.max_message_size, &state.sessions, &state.sink
                    ).await {
                        HandleResult::Ok => {}
                        HandleResult::Disconnected => {
                            router_reconnect(&maybe_connector, &config, &state, &endpoint).await;
                            continue 'router;
                        }
                    }
                }
                result = async { state.stream.write().await.next().await } => {
                    let Some(result) = result else {
                        router_reconnect(&maybe_connector, &config, &state, &endpoint).await;
                        continue 'router;
                    };

                    match result {
                        Ok(message) => {
                            match handle_response::<Message, _, _>(
                                &message, config.max_message_size, &state.sessions, &state.sink
                            ).await {
                                HandleResult::Ok => continue,
                                HandleResult::Disconnected => {
                                    router_reconnect(&maybe_connector, &config, &state, &endpoint).await;
                                    continue 'router;
                                }
                            }
                        }
                        Err(error) => {
                            reset_sessions(&state.sessions).await;
                            match error {
                                WsError::ConnectionClosed => {
                                    trace!("Connection successfully closed on the server");
                                }
                                _ => {
                                    trace!("Native WebSocket stream failed");
                                }
                            }
                            router_reconnect(&maybe_connector, &config, &state, &endpoint).await;
                            continue 'router;
                        }
                    }
                }
                _ = pinger.next() => {
                    trace!("Pinging the server");
                    if state.sink.write().await.send(ping.clone()).await.is_err() {
                        trace!("Native WebSocket ping transmission failed");
                        router_reconnect(&maybe_connector, &config, &state, &endpoint).await;
                        continue 'router;
                    }
                }
            }
        }
    }
}

/// Finite native profile. Socket ownership stays lexical so a failed transport
/// is actually dropped before pending correlations/admission are released.
async fn run_bounded_router(
    endpoint: Endpoint,
    connector: Option<Connector>,
    config: WebSocketConfig,
    initial_socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    routes: Receiver<Route>,
    sessions_rx: Receiver<SessionId>,
) {
    let sessions: Sessions = HashMap::new();
    let mut socket = Some(initial_socket);
    let mut backlog = Vec::<Route>::new();
    loop {
        if routes.is_closed() {
            break;
        }
        let current = if let Some(socket) = socket.take() {
            socket
        } else {
            let attempt = time::timeout(
                time::Duration::from_secs(5),
                connect(&endpoint, Some(config), connector.clone()),
            );
            tokio::pin!(attempt);
            let mut poll = time::interval(time::Duration::from_millis(10));
            loop {
                tokio::select! { biased;
                    _ = poll.tick() => {
                        if routes.is_closed() { break; }
                        backlog.retain(|route| {
                            if let Some(context) = &route.request.request_context {
                                if context.is_interrupted() {
                                    let _ = route.response.try_send(Err(context.interruption_error()));
                                    return false;
                                }
                            }
                            !route.response.is_closed()
                        });
                    }
                    result = &mut attempt => { socket = result.ok().and_then(Result::ok); break; }
                    route = routes.recv(), if backlog.len() < 34 => {
                        if let Ok(route) = route { backlog.push(route); } else { break; }
                    }
                }
            }
            if routes.is_closed() {
                break;
            }
            if let Some(socket) = socket.take() {
                socket
            } else {
                time::sleep(time::Duration::from_millis(100)).await;
                continue;
            }
        };
        {
            let (sink, mut stream) = current.split();
            let sink = RwLock::new(sink);
            let mut maintenance = time::interval(time::Duration::from_millis(10));
            maintenance.set_missed_tick_behavior(MissedTickBehavior::Delay);
            let mut ping = time::interval(PING_INTERVAL);
            let replay = async {
                for (id, state) in sessions.to_vec() {
                    if let Ok(state) = state {
                        replay_session::<Message, _, _>(id, &state, &sink).await?;
                    }
                }
                Ok::<(), Error>(())
            };
            if matches!(
                time::timeout(time::Duration::from_secs(5), replay).await,
                Ok(Ok(()))
            ) {
                loop {
                    tokio::select! { biased;
                        _ = maintenance.tick() => {
                            if routes.is_closed() || super::maintain_request_lifetimes(&sessions).await { break; }
                        }
                        event = sessions_rx.recv() => {
                            let Ok(event) = event else { routes.close(); break; };
                            if !bounded_session_event(event, &sessions, &sink, &routes).await { break; }
                        }
                        route = async {
                            if !backlog.is_empty() { Ok(backlog.remove(0)) } else { routes.recv().await }
                        } => {
                            let Ok(route) = route else { break; };
                            let mut alive = true;
                            while let Ok(event) = sessions_rx.try_recv() {
                                if !bounded_session_event(event, &sessions, &sink, &routes).await { alive = false; break; }
                            }
                            if !alive { backlog.push(route); break; }
                            if !matches!(time::timeout(time::Duration::from_secs(5), handle_route::<Message, _, _>(route, config.max_message_size, &sessions, &sink)).await, Ok(HandleResult::Ok)) { break; }
                        }
                        message = stream.next() => {
                            let Some(Ok(message)) = message else { break; };
                            if !matches!(time::timeout(time::Duration::from_secs(5), handle_response::<Message, _, _>(&message, config.max_message_size, &sessions, &sink)).await, Ok(HandleResult::Ok)) { break; }
                        }
                        _ = ping.tick() => {
                            if !matches!(time::timeout(time::Duration::from_secs(5), sink.write().await.send(create_ping_message())).await, Ok(Ok(()))) { break; }
                        }
                    }
                }
            }
        } // Both split halves dropped here, before local admission is released.
        reset_sessions(&sessions).await;
    }
    reset_sessions(&sessions).await;
    while let Ok(route) = routes.try_recv() {
        backlog.push(route);
    }
    for route in backlog {
        let _ = route.response.try_send(Err(Error::connection(
            "Physical connection closed".to_string(),
            ConnectionError::ConnectionFailed,
        )));
    }
}

async fn bounded_session_event(
    event: SessionId,
    sessions: &Sessions,
    sink: &RwLock<MessageSink>,
    routes: &Receiver<Route>,
) -> bool {
    match event {
        SessionId::Initial(id) if sessions.is_empty() || sessions.contains_key(&id) => {}
        SessionId::Drop(id) if sessions.contains_key(&id) => {
            // Keep pending correlation/admission until the lexical socket owner
            // drops both halves and resets the session, even on ordinary Drop.
            routes.close();
            return false;
        }
        SessionId::Drop(_) => return true,
        _ => return true, // Refuse additional sessions without allocating a registry entry.
    }
    if time::timeout(
        time::Duration::from_secs(5),
        handle_session::<Message, _, _>(event, sessions, sink),
    )
    .await
    .is_err()
    {
        return false;
    }
    for (_, state) in sessions.to_vec() {
        if let Ok(state) = state {
            state
                .bounded
                .store(true, std::sync::atomic::Ordering::Release);
        }
    }
    true
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::Compression;
    use flate2::write::GzEncoder;
    use rand::Rng;
    use web_time::SystemTime;

    use crate::types::{Array, Value};

    #[test_log::test]
    fn large_vector_serialisation_bench() {
        let timed = |func: &dyn Fn() -> Vec<u8>| {
            let start = SystemTime::now();
            let r = func();
            (start.elapsed().unwrap(), r)
        };
        let compress = |v: &Vec<u8>| {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(v).unwrap();
            encoder.finish().unwrap()
        };
        let vector_size = if cfg!(debug_assertions) {
            200_000
        } else {
            2_000_000
        };
        let mut vector: Vec<i32> = Vec::new();
        let mut rng = rand::rng();
        for _ in 0..vector_size {
            vector.push(rng.random());
        }
        let mut results = vec![];

        let vector = Value::Array(Array::from(vector));

        const FLATBUFFERS: &str = "Flatbuffers Vec<Value>";
        const FLATBUFFERS_COMPRESSED: &str = "Flatbuffers Compressed Vec<Value>";
        {
            let (duration, payload) = timed(&|| surrealdb_types::encode(&vector).unwrap());
            results.push((payload.len(), FLATBUFFERS, duration, 1.0));

            let (compression_duration, payload) = timed(&|| compress(&payload));
            let duration = duration + compression_duration;
            results.push((payload.len(), FLATBUFFERS_COMPRESSED, duration, 1.0));
        }

        results.sort_by_key(|(a, _, _, _)| *a);
        for (size, name, duration, factor) in &results {
            info!("{name} - Size: {size} - Duration: {duration:?} - Factor: {factor}");
        }

        let results: Vec<&str> = results.into_iter().map(|(_, name, _, _)| name).collect();
        assert_eq!(results, vec![FLATBUFFERS_COMPRESSED, FLATBUFFERS])
    }
}
