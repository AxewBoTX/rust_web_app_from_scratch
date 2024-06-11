//! # browzer_web
//!
//! `browzer_web` is a very simple framework for building web applications and backends.
//!
//! ## Examples
//!
//! ```rust
//! use browzer_web;
//!
//! fn main() {
//!     let mut server = browzer_web::WebServer::new(format!("0.0.0.0:{}", PORT), 5);
//!     server.get("/", |mut c| {
//!         return c.send_string(browzer_web::response::HttpStatusCode::OK, "Hello, World!");
//!     });
//!     server.listen();
//! }
//! ```
//!
//! ## Modules
//!
//! - `context`: route context which helps to easily work with router handlers
//! - `error`: custom errors
//! - `request`: handle HTTP requests related functionality
//! - `router`: deals with routing and other aspects of routing like middlewares, registered routes
//! - `utils`: utilities used by the framework

pub mod context;
pub mod error;
pub mod request;
pub mod response;
pub mod router;
pub mod utils;

// standard library imports
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
};

/// Represents a web server.
///
/// The `WebServer` struct is responsible for creating the main server which binds all the
/// functionality of the web framework like routing, response generation, listening to
/// requests, etc together efficiently and properly.
///
/// # Fields
///
/// - `listener` - A `TcpListener` that listens for incoming requests streams.
/// - `request_pool`- A custom `ThreadPool` implementation which handles request distribution to various worker threads
/// - `hide_banner` - A boolean flag to control whether the server banner should be displayed(logged to the console) or not
/// - `address` - The address to which the WebServer binds the TcpListener
/// - `router` - An `Arc` wrapped `WebRouter` which is responsible for routing logic of the server
///
/// # Examples
///
/// ```rust
/// use browzer_web::WebServer;
///
/// let server = WebServer::new("127.0.0.1:8080".to_string(), 4);
/// server.listen();
/// ```
// ----- WebServer struct
#[derive(Debug)]
pub struct WebServer {
    pub listener: TcpListener,
    request_pool: utils::thread_pool::ThreadPool,
    pub hide_banner: bool,
    pub address: String,
    router: Arc<router::WebRouter>,
}

impl WebServer {
    /// Creates a new `WebServer` instance.
    ///
    /// Create a `TcpListener`, bind it to the address provided, create a `ThreadPool` with
    /// user-defined number of workers which handles distribution of requests to worker threads and
    /// return the `WebServer` object.
    ///
    /// # Arguments
    ///
    /// - `address` - A `String` representing the address on which the server will listen for
    /// incoming requests.
    /// - `workers` - A `usize` specifying the  number of worker threads that will be created in
    /// the thread pool, to which the incoming requets will be distributed.
    ///
    /// # Returns
    ///
    /// - `WebServer` - A new instance of `WebServer`.
    ///
    /// # Panics
    ///
    /// This function will panic if it fails to bind the `TcpListener` to the provided address.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use browzer_web::WebServer;
    ///
    /// let server = WebServer::new("127.0.0.1:8080".to_string(), 4);
    /// server.listen();
    /// ```
    pub fn new(address: String, workers: usize) -> WebServer {
        let listener = match TcpListener::bind(&address) {
            Ok(listener) => listener,
            Err(listener_create_err) => {
                panic!(
                    "Failed to create listener for the WebServer, Error: {}",
                    listener_create_err.to_string()
                );
            }
        };

        let request_pool = utils::thread_pool::ThreadPool::new(workers);

        // return the WebServer struct
        return WebServer {
            listener,
            request_pool,
            hide_banner: false,
            address,
            router: Arc::new(router::WebRouter::new()),
        };
    }

    /// Registers a new route for handling HTTP GET requests.
    ///
    /// This method allows you to define a route and associate it with a handler function that
    /// will be called when a GET request is made to the specified path. The handler function
    /// should accept a `Context` object and return a `Response` object.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice that holds the path for the route. This is the URL path that will be
    ///   matched against incoming GET requests.
    /// * `handler` - A closure or function that takes a `Context` as input and returns a `Response`.
    ///   The handler function must be `'static`, `Send`, and `Sync`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut server = WebServer::new("127.0.0.1:8080".to_string(), 4);
    ///
    /// server.get("/hello", |mut ctx| {
    ///     return ctx.send_string(browzer_web::response::HttpStatusCode::OK, "Hello, World!");
    /// });
    /// ```
    ///
    /// # Errors
    ///
    /// If the router is not initialized, this method will print an error message using `eprintln!`.
    ///
    /// # Panics
    ///
    /// This function will not panic under normal conditions. However, if the router is not properly
    /// initialized, it will log an error.
    // ----- GET request
    pub fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(context::Context) -> response::Response + 'static + Send + Sync,
    {
        match Arc::get_mut(&mut self.router) {
            Some(router) => router.add(
                path.to_string(),
                utils::HttpMethod::GET,
                router::RouteHandler::new(handler),
            ),
            None => eprintln!(
                "{}",
                error::WebServerError::InternalServerError(
                    "WebRouter is not innitialized".to_string()
                )
            ),
        };
    }
    /// Registers a new route for handling HTTP POST requests.
    ///
    /// This method allows you to define a route and associate it with a handler function that
    /// will be called when a POST request is made to the specified path. The handler function
    /// should accept a `Context` object and return a `Response` object.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice that holds the path for the route. This is the URL path that will be
    ///   matched against incoming POST requests.
    /// * `handler` - A closure or function that takes a `Context` as input and returns a `Response`.
    ///   The handler function must be `'static`, `Send`, and `Sync`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut server = WebServer::new("127.0.0.1:8080".to_string(), 4);
    ///
    /// server.post("/submit", |mut ctx| {
    ///     return ctx.send_string(browzer_web::response::HttpStatusCode::OK, "Resource submitted!");
    /// });
    /// ```
    ///
    /// # Errors
    ///
    /// If the router is not initialized, this method will print an error message using `eprintln!`.
    ///
    /// # Panics
    ///
    /// This function will not panic under normal conditions. However, if the router is not properly
    /// initialized, it will log an error.
    // ----- POST request
    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(context::Context) -> response::Response + 'static + Send + Sync,
    {
        match Arc::get_mut(&mut self.router) {
            Some(router) => router.add(
                path.to_string(),
                utils::HttpMethod::POST,
                router::RouteHandler::new(handler),
            ),
            None => eprintln!(
                "{}",
                error::WebServerError::InternalServerError(
                    "WebRouter is not innitialized".to_string()
                )
            ),
        };
    }
    /// Registers a new route for handling HTTP PATCH requests.
    ///
    /// This method allows you to define a route and associate it with a handler function that
    /// will be called when a PATCH request is made to the specified path. The handler function
    /// should accept a `Context` object and return a `Response` object.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice that holds the path for the route. This is the URL path that will be
    ///   matched against incoming PATCH requests.
    /// * `handler` - A closure or function that takes a `Context` as input and returns a `Response`.
    ///   The handler function must be `'static`, `Send`, and `Sync`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut server = WebServer::new("127.0.0.1:8080".to_string(), 4);
    ///
    /// server.patch("/update", |mut ctx| {
    ///     return ctx.send_string(browzer_web::response::HttpStatusCode::OK, "Resource patched!");
    /// });
    /// ```
    ///
    /// # Errors
    ///
    /// If the router is not initialized, this method will print an error message using `eprintln!`.
    ///
    /// # Panics
    ///
    /// This function will not panic under normal conditions. However, if the router is not properly
    /// initialized, it will log an error.
    // ----- PATCH request
    pub fn patch<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(context::Context) -> response::Response + 'static + Send + Sync,
    {
        match Arc::get_mut(&mut self.router) {
            Some(router) => router.add(
                path.to_string(),
                utils::HttpMethod::PATCH,
                router::RouteHandler::new(handler),
            ),
            None => eprintln!(
                "{}",
                error::WebServerError::InternalServerError(
                    "WebRouter is not innitialized".to_string()
                )
            ),
        };
    }
    /// Registers a new route for handling HTTP DELETE requests.
    ///
    /// This method allows you to define a route and associate it with a handler function that
    /// will be called when a DELETE request is made to the specified path. The handler function
    /// should accept a `Context` object and return a `Response` object.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice that holds the path for the route. This is the URL path that will be
    ///   matched against incoming DELETE requests.
    /// * `handler` - A closure or function that takes a `Context` as input and returns a `Response`.
    ///   The handler function must be `'static`, `Send`, and `Sync`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut server = WebServer::new("127.0.0.1:8080".to_string(), 4);
    ///
    /// server.delete("/remove", |mut ctx|{
    ///     return ctx.send_string(browzer_web::response::HttpStatusCode::OK, "Resource deleted!");
    /// });
    /// ```
    ///
    /// # Errors
    ///
    /// If the router is not initialized, this method will print an error message using `eprintln!`.
    ///
    /// # Panics
    ///
    /// This function will not panic under normal conditions. However, if the router is not properly
    /// initialized, it will log an error.
    // ----- DELETE request
    pub fn delete<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(context::Context) -> response::Response + 'static + Send + Sync,
    {
        match Arc::get_mut(&mut self.router) {
            Some(router) => router.add(
                path.to_string(),
                utils::HttpMethod::DELETE,
                router::RouteHandler::new(handler),
            ),
            None => eprintln!(
                "{}",
                error::WebServerError::InternalServerError(
                    "WebRouter is not innitialized".to_string()
                )
            ),
        };
    }

    /// Listens for incoming TCP connections and handles them using the web server.
    ///
    /// This function starts the web server, accepting incoming connections and distributing
    /// them to worker threads for handling. It uses the `request_pool` to manage a pool of
    /// worker threads and assigns incoming requests to these workers. The function will
    /// continue to listen for connections indefinitely.
    ///
    /// # Panics
    ///
    /// This function will not panic under normal conditions. However, it will print error
    /// messages to the standard error output if it encounters issues with establishing connections
    /// or assigning worker threads.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut server = WebServer::new("127.0.0.1:8080".to_string(), 4);
    /// server.listen();
    /// ```
    ///
    /// This example demonstrates how to start the web server and listen for incoming connections.
    pub fn listen(&self) {
        // print the server banner( a simple log message ) accoding to the `address` field boolean variable
        if !self.hide_banner {
            println!("-----> HTTP server running on {}", self.address);
        }

        // loop over incoming requests and send those request as jobs to the `request_pool` in
        // order to be distributed to the worker threads
        for stream in self.listener.incoming() {
            let router = Arc::clone(&self.router);
            match stream {
                Ok(stream) => {
                    match self.request_pool.execute(|| {
                        match Self::handle_request(router, stream) {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("Failed to handle incoming request, Error: {}", e);
                            }
                        };
                    }) {
                        Ok(_) => {}
                        Err(e) => eprintln!(
                            "Failed to assign Worker thread to incoming request, Error: {}",
                            e.to_string()
                        ),
                    };
                }
                Err(e) => {
                    eprintln!("Failed to establish a connection, Error: {}", e.to_string());
                }
            }
        }
    }

    // handles various operations related to incoming requests.
    fn handle_request(
        router: Arc<router::WebRouter>,
        mut stream: TcpStream,
    ) -> Result<(), error::WebServerError> {
        let buf_reader = BufReader::new(&mut stream);

        // parse the request string into a `Request` struct by first parsing the string to a string
        // vector containling the lines of requests as elements and then passing that vector onto the
        // `new` function of the `Request` string as input
        let request = match request::Request::new(&match buf_reader
            .lines()
            .take_while(|result| match result {
                Ok(line) => !line.is_empty(),
                Err(_) => false,
            })
            .collect()
        {
            Ok(request) => request,
            Err(e) => return Err(error::WebServerError::IO(e)),
        }) {
            Ok(safe) => safe,
            Err(e) => {
                return Err(error::WebServerError::RequestParseError(e));
            }
        };

        // utilize user registered routes from `routes` hashmap in the `WebRouter` to handle
        // requests, generate responses and then send those responses to the request agent throught
        // the TCP connection stream
        match stream.write_all(router.handle_request(request).to_string().as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                return Err(error::WebServerError::IO(e));
            }
        };

        match stream.flush() {
            Ok(_) => Ok({}),
            Err(e) => {
                return Err(error::WebServerError::StreamFlushError(e.to_string()));
            }
        }
    }
}