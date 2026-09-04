//references: https://github.com/cmackenzie1/torii-rs/blob/main/torii-client/src/lib.rs

mod auth;

pub use auth::{ApiErrorResponse, RegisterDto, RegisterRequest, RegisterUserResponse};

use serde::{Serialize, de::DeserializeOwned};

pub mod prefixes {
    pub const AUTH: &str = "/auth";
}

pub mod path {
    pub mod auth {
        pub const REGISTER: &str = "/auth/register";
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Delete,
}

#[derive(Debug, Clone, Copy)]
pub struct EndpointContract<Request, Response> {
    pub path: &'static str,
    pub method: Method,
    _contract: std::marker::PhantomData<fn(Request) -> Response>,
}

impl<Request, Response> EndpointContract<Request, Response> {
    pub const fn new(path: &'static str, method: Method) -> Self {
        Self {
            path,
            method,
            _contract: std::marker::PhantomData,
        }
    }
}

pub trait Endpoint {
    type Request: Serialize;
    type Response: DeserializeOwned;
    const CONTRACT: EndpointContract<Self::Request, Self::Response>;
}

pub trait EndpointRequest: Serialize {
    type Endpoint: Endpoint<Request = Self>;
}

#[macro_export]
macro_rules! define_endpoint {
    (
        $name:ident,
        $method:expr,
        $path:expr,
        $request:ty,
        $response:ty
    ) => {
        pub struct $name;

        impl $crate::Endpoint for $name {
            type Request = $request;
            type Response = $response;

            const CONTRACT: $crate::EndpointContract<Self::Request, Self::Response> =
                $crate::EndpointContract::new($path, $method);
        }
    };
}

define_endpoint!(
    RegisterEndpoint,
    Method::Post,
    path::auth::REGISTER,
    RegisterRequest,
    RegisterDto
);
