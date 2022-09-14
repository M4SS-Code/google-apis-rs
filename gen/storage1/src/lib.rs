// DO NOT EDIT !
// This file was generated automatically from 'src/generator/templates/api/lib.rs.mako'
// DO NOT EDIT !

//! This documentation was generated from *storage* crate version *4.0.1+20220228*, where *20220228* is the exact revision of the *storage:v1* schema built by the [mako](http://www.makotemplates.org/) code generator *v4.0.1*.
//! 
//! Everything else about the *storage* *v1* API can be found at the
//! [official documentation site](https://developers.google.com/storage/docs/json_api/).
//! The original source code is [on github](https://github.com/Byron/google-apis-rs/tree/main/gen/storage1).
//! # Features
//! 
//! Handle the following *Resources* with ease from the central [hub](Storage) ... 
//! 
//! * [bucket access controls](api::BucketAccessControl)
//!  * [*delete*](api::BucketAccessControlDeleteCall), [*get*](api::BucketAccessControlGetCall), [*insert*](api::BucketAccessControlInsertCall), [*list*](api::BucketAccessControlListCall), [*patch*](api::BucketAccessControlPatchCall) and [*update*](api::BucketAccessControlUpdateCall)
//! * [buckets](api::Bucket)
//!  * [*delete*](api::BucketDeleteCall), [*get*](api::BucketGetCall), [*get iam policy*](api::BucketGetIamPolicyCall), [*insert*](api::BucketInsertCall), [*list*](api::BucketListCall), [*lock retention policy*](api::BucketLockRetentionPolicyCall), [*patch*](api::BucketPatchCall), [*set iam policy*](api::BucketSetIamPolicyCall), [*test iam permissions*](api::BucketTestIamPermissionCall) and [*update*](api::BucketUpdateCall)
//! * [channels](api::Channel)
//!  * [*stop*](api::ChannelStopCall)
//! * default object access controls
//!  * [*delete*](api::DefaultObjectAccessControlDeleteCall), [*get*](api::DefaultObjectAccessControlGetCall), [*insert*](api::DefaultObjectAccessControlInsertCall), [*list*](api::DefaultObjectAccessControlListCall), [*patch*](api::DefaultObjectAccessControlPatchCall) and [*update*](api::DefaultObjectAccessControlUpdateCall)
//! * [notifications](api::Notification)
//!  * [*delete*](api::NotificationDeleteCall), [*get*](api::NotificationGetCall), [*insert*](api::NotificationInsertCall) and [*list*](api::NotificationListCall)
//! * [object access controls](api::ObjectAccessControl)
//!  * [*delete*](api::ObjectAccessControlDeleteCall), [*get*](api::ObjectAccessControlGetCall), [*insert*](api::ObjectAccessControlInsertCall), [*list*](api::ObjectAccessControlListCall), [*patch*](api::ObjectAccessControlPatchCall) and [*update*](api::ObjectAccessControlUpdateCall)
//! * [objects](api::Object)
//!  * [*compose*](api::ObjectComposeCall), [*copy*](api::ObjectCopyCall), [*delete*](api::ObjectDeleteCall), [*get*](api::ObjectGetCall), [*get iam policy*](api::ObjectGetIamPolicyCall), [*insert*](api::ObjectInsertCall), [*list*](api::ObjectListCall), [*patch*](api::ObjectPatchCall), [*rewrite*](api::ObjectRewriteCall), [*set iam policy*](api::ObjectSetIamPolicyCall), [*test iam permissions*](api::ObjectTestIamPermissionCall), [*update*](api::ObjectUpdateCall) and [*watch all*](api::ObjectWatchAllCall)
//! * projects
//!  * [*hmac keys create*](api::ProjectHmacKeyCreateCall), [*hmac keys delete*](api::ProjectHmacKeyDeleteCall), [*hmac keys get*](api::ProjectHmacKeyGetCall), [*hmac keys list*](api::ProjectHmacKeyListCall), [*hmac keys update*](api::ProjectHmacKeyUpdateCall) and [*service account get*](api::ProjectServiceAccountGetCall)
//! 
//! 
//! Upload supported by ...
//! 
//! * [*insert objects*](api::ObjectInsertCall)
//! 
//! Download supported by ...
//! 
//! * [*get objects*](api::ObjectGetCall)
//! 
//! Subscription supported by ...
//! 
//! * [*list objects*](api::ObjectListCall)
//! * [*watch all objects*](api::ObjectWatchAllCall)
//! 
//! 
//! 
//! Not what you are looking for ? Find all other Google APIs in their Rust [documentation index](http://byron.github.io/google-apis-rs).
//! 
//! # Structure of this Library
//! 
//! The API is structured into the following primary items:
//! 
//! * **[Hub](Storage)**
//!     * a central object to maintain state and allow accessing all *Activities*
//!     * creates [*Method Builders*](client::MethodsBuilder) which in turn
//!       allow access to individual [*Call Builders*](client::CallBuilder)
//! * **[Resources](client::Resource)**
//!     * primary types that you can apply *Activities* to
//!     * a collection of properties and *Parts*
//!     * **[Parts](client::Part)**
//!         * a collection of properties
//!         * never directly used in *Activities*
//! * **[Activities](client::CallBuilder)**
//!     * operations to apply to *Resources*
//! 
//! All *structures* are marked with applicable traits to further categorize them and ease browsing.
//! 
//! Generally speaking, you can invoke *Activities* like this:
//! 
//! ```Rust,ignore
//! let r = hub.resource().activity(...).doit().await
//! ```
//! 
//! Or specifically ...
//! 
//! ```ignore
//! let r = hub.objects().compose(...).doit().await
//! let r = hub.objects().copy(...).doit().await
//! let r = hub.objects().delete(...).doit().await
//! let r = hub.objects().get(...).doit().await
//! let r = hub.objects().get_iam_policy(...).doit().await
//! let r = hub.objects().insert(...).doit().await
//! let r = hub.objects().list(...).doit().await
//! let r = hub.objects().patch(...).doit().await
//! let r = hub.objects().rewrite(...).doit().await
//! let r = hub.objects().set_iam_policy(...).doit().await
//! let r = hub.objects().test_iam_permissions(...).doit().await
//! let r = hub.objects().update(...).doit().await
//! let r = hub.objects().watch_all(...).doit().await
//! ```
//! 
//! The `resource()` and `activity(...)` calls create [builders][builder-pattern]. The second one dealing with `Activities` 
//! supports various methods to configure the impending operation (not shown here). It is made such that all required arguments have to be 
//! specified right away (i.e. `(...)`), whereas all optional ones can be [build up][builder-pattern] as desired.
//! The `doit()` method performs the actual communication with the server and returns the respective result.
//! 
//! # Usage
//! 
//! ## Setting up your Project
//! 
//! To use this library, you would put the following lines into your `Cargo.toml` file:
//! 
//! ```toml
//! [dependencies]
//! google-storage1 = "*"
//! serde = "^1.0"
//! serde_json = "^1.0"
//! ```
//! 
//! ## A complete example
//! 
//! ```test_harness,no_run
//! extern crate hyper;
//! extern crate hyper_rustls;
//! extern crate google_storage1 as storage1;
//! use storage1::api::Object;
//! use storage1::{Result, Error};
//! # async fn dox() {
//! use std::default::Default;
//! use storage1::{Storage, oauth2, hyper, hyper_rustls};
//! 
//! // Get an ApplicationSecret instance by some means. It contains the `client_id` and 
//! // `client_secret`, among other things.
//! let secret: oauth2::ApplicationSecret = Default::default();
//! // Instantiate the authenticator. It will choose a suitable authentication flow for you, 
//! // unless you replace  `None` with the desired Flow.
//! // Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about 
//! // what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
//! // retrieve them from storage.
//! let auth = oauth2::InstalledFlowAuthenticator::builder(
//!         secret,
//!         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
//!     ).build().await.unwrap();
//! let mut hub = Storage::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
//! // As the method needs a request, you would usually fill it with the desired information
//! // into the respective structure. Some of the parts shown here might not be applicable !
//! // Values shown here are possibly random and not representative !
//! let mut req = Object::default();
//! 
//! // You can configure optional parameters by calling the respective setters at will, and
//! // execute the final call using `doit()`.
//! // Values shown here are possibly random and not representative !
//! let result = hub.objects().rewrite(req, "sourceBucket", "sourceObject", "destinationBucket", "destinationObject")
//!              .user_project("sed")
//!              .source_generation("ut")
//!              .rewrite_token("gubergren")
//!              .provisional_user_project("rebum.")
//!              .projection("est")
//!              .max_bytes_rewritten_per_call("ipsum")
//!              .if_source_metageneration_not_match("ipsum")
//!              .if_source_metageneration_match("est")
//!              .if_source_generation_not_match("gubergren")
//!              .if_source_generation_match("ea")
//!              .if_metageneration_not_match("dolor")
//!              .if_metageneration_match("Lorem")
//!              .if_generation_not_match("eos")
//!              .if_generation_match("labore")
//!              .destination_predefined_acl("sed")
//!              .destination_kms_key_name("duo")
//!              .doit().await;
//! 
//! match result {
//!     Err(e) => match e {
//!         // The Error enum provides details about what exactly happened.
//!         // You can also just use its `Debug`, `Display` or `Error` traits
//!          Error::HttpError(_)
//!         |Error::Io(_)
//!         |Error::MissingAPIKey
//!         |Error::MissingToken(_)
//!         |Error::Cancelled
//!         |Error::UploadSizeLimitExceeded(_, _)
//!         |Error::Failure(_)
//!         |Error::BadRequest(_)
//!         |Error::FieldClash(_)
//!         |Error::JsonDecodeError(_, _) => println!("{}", e),
//!     },
//!     Ok(res) => println!("Success: {:?}", res),
//! }
//! # }
//! ```
//! ## Handling Errors
//! 
//! All errors produced by the system are provided either as [Result](client::Result) enumeration as return value of
//! the doit() methods, or handed as possibly intermediate results to either the 
//! [Hub Delegate](client::Delegate), or the [Authenticator Delegate](https://docs.rs/yup-oauth2/*/yup_oauth2/trait.AuthenticatorDelegate.html).
//! 
//! When delegates handle errors or intermediate values, they may have a chance to instruct the system to retry. This 
//! makes the system potentially resilient to all kinds of errors.
//! 
//! ## Uploads and Downloads
//! If a method supports downloads, the response body, which is part of the [Result](client::Result), should be
//! read by you to obtain the media.
//! If such a method also supports a [Response Result](client::ResponseResult), it will return that by default.
//! You can see it as meta-data for the actual media. To trigger a media download, you will have to set up the builder by making
//! this call: `.param("alt", "media")`.
//! 
//! Methods supporting uploads can do so using up to 2 different protocols: 
//! *simple* and *resumable*. The distinctiveness of each is represented by customized 
//! `doit(...)` methods, which are then named `upload(...)` and `upload_resumable(...)` respectively.
//! 
//! ## Customization and Callbacks
//! 
//! You may alter the way an `doit()` method is called by providing a [delegate](client::Delegate) to the 
//! [Method Builder](client::CallBuilder) before making the final `doit()` call. 
//! Respective methods will be called to provide progress information, as well as determine whether the system should 
//! retry on failure.
//! 
//! The [delegate trait](client::Delegate) is default-implemented, allowing you to customize it with minimal effort.
//! 
//! ## Optional Parts in Server-Requests
//! 
//! All structures provided by this library are made to be [encodable](client::RequestValue) and 
//! [decodable](client::ResponseResult) via *json*. Optionals are used to indicate that partial requests are responses 
//! are valid.
//! Most optionals are are considered [Parts](client::Part) which are identifiable by name, which will be sent to 
//! the server to indicate either the set parts of the request or the desired parts in the response.
//! 
//! ## Builder Arguments
//! 
//! Using [method builders](client::CallBuilder), you are able to prepare an action call by repeatedly calling it's methods.
//! These will always take a single argument, for which the following statements are true.
//! 
//! * [PODs][wiki-pod] are handed by copy
//! * strings are passed as `&str`
//! * [request values](client::RequestValue) are moved
//! 
//! Arguments will always be copied or cloned into the builder, to make them independent of their original life times.
//! 
//! [wiki-pod]: http://en.wikipedia.org/wiki/Plain_old_data_structure
//! [builder-pattern]: http://en.wikipedia.org/wiki/Builder_pattern
//! [google-go-api]: https://github.com/google/google-api-go-client
//! 
//! 

// Unused attributes happen thanks to defined, but unused structures
// We don't warn about this, as depending on the API, some data structures or facilities are never used.
// Instead of pre-determining this, we just disable the lint. It's manually tuned to not have any
// unused imports in fully featured APIs. Same with unused_mut ... .
#![allow(unused_imports, unused_mut, dead_code)]

// DO NOT EDIT !
// This file was generated automatically from 'src/generator/templates/api/lib.rs.mako'
// DO NOT EDIT !

#[macro_use]
extern crate serde_derive;

// Re-export the hyper and hyper_rustls crate, they are required to build the hub
pub extern crate hyper;
pub extern crate hyper_rustls;
extern crate serde;
extern crate serde_json;
// Re-export the yup_oauth2 crate, that is required to call some methods of the hub and the client
pub extern crate yup_oauth2 as oauth2;
extern crate mime;
extern crate url;

pub mod api;
pub mod client;

// Re-export the hub type and some basic client structs
pub use api::Storage;
pub use client::{Result, Error, Delegate};
