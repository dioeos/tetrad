//references: https://github.com/cmackenzie1/torii-rs/blob/main/torii-client/src/lib.rs

mod auth;
mod error;

pub use auth::{LoginRequest, LoginUserDto, RegisterDto, RegisterRequest, RegisterUserResponse};
pub use error::ApiErrorResponse;

use serde::{Serialize, de::DeserializeOwned};

pub mod path {
    pub mod auth {
        pub const REGISTER: &str = "/auth/register";
        pub const LOGIN: &str = "/auth/login";
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

define_endpoint!(
    LoginEndpoint,
    Method::Post,
    path::auth::LOGIN,
    LoginRequest,
    LoginUserDto
);
