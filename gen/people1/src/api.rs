use std::collections::HashMap;
use std::cell::RefCell;
use std::default::Default;
use std::collections::BTreeMap;
use std::error::Error as StdError;
use serde_json as json;
use std::io;
use std::fs;
use std::mem;
use std::thread::sleep;

use http::Uri;
use hyper::client::connect;
use tokio::io::{AsyncRead, AsyncWrite};
use tower_service;
use crate::client;

// ##############
// UTILITIES ###
// ############

/// Identifies the an OAuth2 authorization scope.
/// A scope is needed when requesting an
/// [authorization token](https://developers.google.com/youtube/v3/guides/authentication).
#[derive(PartialEq, Eq, Hash)]
pub enum Scope {
    /// See, edit, download, and permanently delete your contacts
    Contact,

    /// See and download contact info automatically saved in your "Other contacts"
    ContactOtherReadonly,

    /// See and download your contacts
    ContactReadonly,

    /// See and download your organization's GSuite directory
    DirectoryReadonly,

    /// View your street addresses
    UserAddresseRead,

    /// See and download your exact date of birth
    UserBirthdayRead,

    /// See and download all of your Google Account email addresses
    UserEmailRead,

    /// See your gender
    UserGenderRead,

    /// See your education, work history and org info
    UserOrganizationRead,

    /// See and download your personal phone numbers
    UserPhonenumberRead,

    /// See your primary Google Account email address
    UserinfoEmail,

    /// See your personal info, including any personal info you've made publicly available
    UserinfoProfile,
}

impl AsRef<str> for Scope {
    fn as_ref(&self) -> &str {
        match *self {
            Scope::Contact => "https://www.googleapis.com/auth/contacts",
            Scope::ContactOtherReadonly => "https://www.googleapis.com/auth/contacts.other.readonly",
            Scope::ContactReadonly => "https://www.googleapis.com/auth/contacts.readonly",
            Scope::DirectoryReadonly => "https://www.googleapis.com/auth/directory.readonly",
            Scope::UserAddresseRead => "https://www.googleapis.com/auth/user.addresses.read",
            Scope::UserBirthdayRead => "https://www.googleapis.com/auth/user.birthday.read",
            Scope::UserEmailRead => "https://www.googleapis.com/auth/user.emails.read",
            Scope::UserGenderRead => "https://www.googleapis.com/auth/user.gender.read",
            Scope::UserOrganizationRead => "https://www.googleapis.com/auth/user.organization.read",
            Scope::UserPhonenumberRead => "https://www.googleapis.com/auth/user.phonenumbers.read",
            Scope::UserinfoEmail => "https://www.googleapis.com/auth/userinfo.email",
            Scope::UserinfoProfile => "https://www.googleapis.com/auth/userinfo.profile",
        }
    }
}

impl Default for Scope {
    fn default() -> Scope {
        Scope::ContactOtherReadonly
    }
}



// ########
// HUB ###
// ######

/// Central instance to access all PeopleService related resource activities
///
/// # Examples
///
/// Instantiate a new hub
///
/// ```test_harness,no_run
/// extern crate hyper;
/// extern crate hyper_rustls;
/// extern crate google_people1 as people1;
/// use people1::{Result, Error};
/// # async fn dox() {
/// use std::default::Default;
/// use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// // Get an ApplicationSecret instance by some means. It contains the `client_id` and 
/// // `client_secret`, among other things.
/// let secret: oauth2::ApplicationSecret = Default::default();
/// // Instantiate the authenticator. It will choose a suitable authentication flow for you, 
/// // unless you replace  `None` with the desired Flow.
/// // Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about 
/// // what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
/// // retrieve them from storage.
/// let auth = oauth2::InstalledFlowAuthenticator::builder(
///         secret,
///         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
///     ).build().await.unwrap();
/// let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.contact_groups().list()
///              .sync_token("amet.")
///              .page_token("takimata")
///              .page_size(-52)
///              .group_fields("duo")
///              .doit().await;
/// 
/// match result {
///     Err(e) => match e {
///         // The Error enum provides details about what exactly happened.
///         // You can also just use its `Debug`, `Display` or `Error` traits
///          Error::HttpError(_)
///         |Error::Io(_)
///         |Error::MissingAPIKey
///         |Error::MissingToken(_)
///         |Error::Cancelled
///         |Error::UploadSizeLimitExceeded(_, _)
///         |Error::Failure(_)
///         |Error::BadRequest(_)
///         |Error::FieldClash(_)
///         |Error::JsonDecodeError(_, _) => println!("{}", e),
///     },
///     Ok(res) => println!("Success: {:?}", res),
/// }
/// # }
/// ```
#[derive(Clone)]
pub struct PeopleService<S> {
    pub client: hyper::Client<S, hyper::body::Body>,
    pub auth: oauth2::authenticator::Authenticator<S>,
    _user_agent: String,
    _base_url: String,
    _root_url: String,
}

impl<'a, S> client::Hub for PeopleService<S> {}

impl<'a, S> PeopleService<S> {

    pub fn new(client: hyper::Client<S, hyper::body::Body>, authenticator: oauth2::authenticator::Authenticator<S>) -> PeopleService<S> {
        PeopleService {
            client,
            auth: authenticator,
            _user_agent: "google-api-rust-client/4.0.1".to_string(),
            _base_url: "https://people.googleapis.com/".to_string(),
            _root_url: "https://people.googleapis.com/".to_string(),
        }
    }

    pub fn contact_groups(&'a self) -> ContactGroupMethods<'a, S> {
        ContactGroupMethods { hub: &self }
    }
    pub fn other_contacts(&'a self) -> OtherContactMethods<'a, S> {
        OtherContactMethods { hub: &self }
    }
    pub fn people(&'a self) -> PeopleMethods<'a, S> {
        PeopleMethods { hub: &self }
    }

    /// Set the user-agent header field to use in all requests to the server.
    /// It defaults to `google-api-rust-client/4.0.1`.
    ///
    /// Returns the previously set user-agent.
    pub fn user_agent(&mut self, agent_name: String) -> String {
        mem::replace(&mut self._user_agent, agent_name)
    }

    /// Set the base url to use in all requests to the server.
    /// It defaults to `https://people.googleapis.com/`.
    ///
    /// Returns the previously set base url.
    pub fn base_url(&mut self, new_base_url: String) -> String {
        mem::replace(&mut self._base_url, new_base_url)
    }

    /// Set the root url to use in all requests to the server.
    /// It defaults to `https://people.googleapis.com/`.
    ///
    /// Returns the previously set root url.
    pub fn root_url(&mut self, new_root_url: String) -> String {
        mem::replace(&mut self._root_url, new_root_url)
    }
}


// ############
// SCHEMAS ###
// ##########
/// A person's physical address. May be a P.O. box or street address. All fields are optional.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Address {
    /// The city of the address.
    pub city: Option<String>,
    /// The country of the address.
    pub country: Option<String>,
    /// The [ISO 3166-1 alpha-2](http://www.iso.org/iso/country_codes.htm) country code of the address.
    #[serde(rename="countryCode")]
    pub country_code: Option<String>,
    /// The extended address of the address; for example, the apartment number.
    #[serde(rename="extendedAddress")]
    pub extended_address: Option<String>,
    /// Output only. The type of the address translated and formatted in the viewer's account locale or the `Accept-Language` HTTP header locale.
    #[serde(rename="formattedType")]
    pub formatted_type: Option<String>,
    /// The unstructured value of the address. If this is not set by the user it will be automatically constructed from structured values.
    #[serde(rename="formattedValue")]
    pub formatted_value: Option<String>,
    /// Metadata about the address.
    pub metadata: Option<FieldMetadata>,
    /// The P.O. box of the address.
    #[serde(rename="poBox")]
    pub po_box: Option<String>,
    /// The postal code of the address.
    #[serde(rename="postalCode")]
    pub postal_code: Option<String>,
    /// The region of the address; for example, the state or province.
    pub region: Option<String>,
    /// The street address.
    #[serde(rename="streetAddress")]
    pub street_address: Option<String>,
    /// The type of the address. The type can be custom or one of these predefined values: * `home` * `work` * `other`
    #[serde(rename="type")]
    pub type_: Option<String>,
}

impl client::Part for Address {}


/// A person's age range.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct AgeRangeType {
    /// The age range.
    #[serde(rename="ageRange")]
    pub age_range: Option<String>,
    /// Metadata about the age range.
    pub metadata: Option<FieldMetadata>,
}

impl client::Part for AgeRangeType {}


/// A request to create a batch of contacts.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [batch create contacts people](PeopleBatchCreateContactCall) (request)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BatchCreateContactsRequest {
    /// Required. The contact to create. Allows up to 200 contacts in a single request.
    pub contacts: Option<Vec<ContactToCreate>>,
    /// Required. A field mask to restrict which fields on each person are returned in the response. Multiple fields can be specified by separating them with commas. If read mask is left empty, the post-mutate-get is skipped and no data will be returned in the response. Valid values are: * addresses * ageRanges * biographies * birthdays * calendarUrls * clientData * coverPhotos * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * metadata * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * photos * relations * sipAddresses * skills * urls * userDefined
    #[serde(rename="readMask")]
    pub read_mask: Option<String>,
    /// Optional. A mask of what source types to return in the post mutate read. Defaults to READ_SOURCE_TYPE_CONTACT and READ_SOURCE_TYPE_PROFILE if not set.
    pub sources: Option<Vec<String>>,
}

impl client::RequestValue for BatchCreateContactsRequest {}


/// If not successful, returns BatchCreateContactsErrorDetails which contains a list of errors for each invalid contact. The response to a request to create a batch of contacts.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [batch create contacts people](PeopleBatchCreateContactCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BatchCreateContactsResponse {
    /// The contacts that were created, unless the request `read_mask` is empty.
    #[serde(rename="createdPeople")]
    pub created_people: Option<Vec<PersonResponse>>,
}

impl client::ResponseResult for BatchCreateContactsResponse {}


/// A request to delete a batch of existing contacts.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [batch delete contacts people](PeopleBatchDeleteContactCall) (request)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BatchDeleteContactsRequest {
    /// Required. The resource names of the contact to delete. It's repeatable. Allows up to 500 resource names in a single request.
    #[serde(rename="resourceNames")]
    pub resource_names: Option<Vec<String>>,
}

impl client::RequestValue for BatchDeleteContactsRequest {}


/// The response to a batch get contact groups request.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [batch get contact groups](ContactGroupBatchGetCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BatchGetContactGroupsResponse {
    /// The list of responses for each requested contact group resource.
    pub responses: Option<Vec<ContactGroupResponse>>,
}

impl client::ResponseResult for BatchGetContactGroupsResponse {}


/// A request to update a batch of contacts.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [batch update contacts people](PeopleBatchUpdateContactCall) (request)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BatchUpdateContactsRequest {
    /// Required. A map of resource names to the person data to be updated. Allows up to 200 contacts in a single request.
    pub contacts: Option<HashMap<String, Person>>,
    /// Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. If read mask is left empty, the post-mutate-get is skipped and no data will be returned in the response. Valid values are: * addresses * ageRanges * biographies * birthdays * calendarUrls * clientData * coverPhotos * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * metadata * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * photos * relations * sipAddresses * skills * urls * userDefined
    #[serde(rename="readMask")]
    pub read_mask: Option<String>,
    /// Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT and READ_SOURCE_TYPE_PROFILE if not set.
    pub sources: Option<Vec<String>>,
    /// Required. A field mask to restrict which fields on the person are updated. Multiple fields can be specified by separating them with commas. All specified fields will be replaced, or cleared if left empty for each person. Valid values are: * addresses * biographies * birthdays * calendarUrls * clientData * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * relations * sipAddresses * urls * userDefined
    #[serde(rename="updateMask")]
    pub update_mask: Option<String>,
}

impl client::RequestValue for BatchUpdateContactsRequest {}


/// If not successful, returns BatchUpdateContactsErrorDetails, a list of errors corresponding to each contact. The response to a request to update a batch of contacts.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [batch update contacts people](PeopleBatchUpdateContactCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BatchUpdateContactsResponse {
    /// A map of resource names to the contacts that were updated, unless the request `read_mask` is empty.
    #[serde(rename="updateResult")]
    pub update_result: Option<HashMap<String, PersonResponse>>,
}

impl client::ResponseResult for BatchUpdateContactsResponse {}


/// A person's short biography.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Biography {
    /// The content type of the biography.
    #[serde(rename="contentType")]
    pub content_type: Option<String>,
    /// Metadata about the biography.
    pub metadata: Option<FieldMetadata>,
    /// The short biography.
    pub value: Option<String>,
}

impl client::Part for Biography {}


/// A person's birthday. At least one of the `date` and `text` fields are specified. The `date` and `text` fields typically represent the same date, but are not guaranteed to. Clients should always set the `date` field when mutating birthdays.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Birthday {
    /// The structured date of the birthday.
    pub date: Option<Date>,
    /// Metadata about the birthday.
    pub metadata: Option<FieldMetadata>,
    /// Prefer to use the `date` field if set. A free-form string representing the user's birthday. This value is not validated.
    pub text: Option<String>,
}

impl client::Part for Birthday {}


/// **DEPRECATED**: No data will be returned A person's bragging rights.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BraggingRights {
    /// Metadata about the bragging rights.
    pub metadata: Option<FieldMetadata>,
    /// The bragging rights; for example, `climbed mount everest`.
    pub value: Option<String>,
}

impl client::Part for BraggingRights {}


/// A person's calendar URL.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CalendarUrl {
    /// Output only. The type of the calendar URL translated and formatted in the viewer's account locale or the `Accept-Language` HTTP header locale.
    #[serde(rename="formattedType")]
    pub formatted_type: Option<String>,
    /// Metadata about the calendar URL.
    pub metadata: Option<FieldMetadata>,
    /// The type of the calendar URL. The type can be custom or one of these predefined values: * `home` * `freeBusy` * `work`
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// The calendar URL.
    pub url: Option<String>,
}

impl client::Part for CalendarUrl {}


/// Arbitrary client data that is populated by clients. Duplicate keys and values are allowed.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ClientData {
    /// The client specified key of the client data.
    pub key: Option<String>,
    /// Metadata about the client data.
    pub metadata: Option<FieldMetadata>,
    /// The client specified value of the client data.
    pub value: Option<String>,
}

impl client::Part for ClientData {}


/// A contact group.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [members modify contact groups](ContactGroupMemberModifyCall) (none)
/// * [batch get contact groups](ContactGroupBatchGetCall) (none)
/// * [create contact groups](ContactGroupCreateCall) (response)
/// * [delete contact groups](ContactGroupDeleteCall) (none)
/// * [get contact groups](ContactGroupGetCall) (response)
/// * [list contact groups](ContactGroupListCall) (none)
/// * [update contact groups](ContactGroupUpdateCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ContactGroup {
    /// The group's client data.
    #[serde(rename="clientData")]
    pub client_data: Option<Vec<GroupClientData>>,
    /// The [HTTP entity tag](https://en.wikipedia.org/wiki/HTTP_ETag) of the resource. Used for web cache validation.
    pub etag: Option<String>,
    /// Output only. The name translated and formatted in the viewer's account locale or the `Accept-Language` HTTP header locale for system groups names. Group names set by the owner are the same as name.
    #[serde(rename="formattedName")]
    pub formatted_name: Option<String>,
    /// Output only. The contact group type.
    #[serde(rename="groupType")]
    pub group_type: Option<String>,
    /// Output only. The total number of contacts in the group irrespective of max members in specified in the request.
    #[serde(rename="memberCount")]
    pub member_count: Option<i32>,
    /// Output only. The list of contact person resource names that are members of the contact group. The field is only populated for GET requests and will only return as many members as `maxMembers` in the get request.
    #[serde(rename="memberResourceNames")]
    pub member_resource_names: Option<Vec<String>>,
    /// Output only. Metadata about the contact group.
    pub metadata: Option<ContactGroupMetadata>,
    /// The contact group name set by the group owner or a system provided name for system groups. For [`contactGroups.create`](/people/api/rest/v1/contactGroups/create) or [`contactGroups.update`](/people/api/rest/v1/contactGroups/update) the name must be unique to the users contact groups. Attempting to create a group with a duplicate name will return a HTTP 409 error.
    pub name: Option<String>,
    /// The resource name for the contact group, assigned by the server. An ASCII string, in the form of `contactGroups/{contact_group_id}`.
    #[serde(rename="resourceName")]
    pub resource_name: Option<String>,
}

impl client::Resource for ContactGroup {}
impl client::ResponseResult for ContactGroup {}


/// A Google contact group membership.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ContactGroupMembership {
    /// Output only. The contact group ID for the contact group membership.
    #[serde(rename="contactGroupId")]
    pub contact_group_id: Option<String>,
    /// The resource name for the contact group, assigned by the server. An ASCII string, in the form of `contactGroups/{contact_group_id}`. Only contact_group_resource_name can be used for modifying memberships. Any contact group membership can be removed, but only user group or "myContacts" or "starred" system groups memberships can be added. A contact must always have at least one contact group membership.
    #[serde(rename="contactGroupResourceName")]
    pub contact_group_resource_name: Option<String>,
}

impl client::Part for ContactGroupMembership {}


/// The metadata about a contact group.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ContactGroupMetadata {
    /// Output only. True if the contact group resource has been deleted. Populated only for [`ListContactGroups`](/people/api/rest/v1/contactgroups/list) requests that include a sync token.
    pub deleted: Option<bool>,
    /// Output only. The time the group was last updated.
    #[serde(rename="updateTime")]
    pub update_time: Option<String>,
}

impl client::Part for ContactGroupMetadata {}


/// The response for a specific contact group.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ContactGroupResponse {
    /// The contact group.
    #[serde(rename="contactGroup")]
    pub contact_group: Option<ContactGroup>,
    /// The original requested resource name.
    #[serde(rename="requestedResourceName")]
    pub requested_resource_name: Option<String>,
    /// The status of the response.
    pub status: Option<Status>,
}

impl client::Part for ContactGroupResponse {}


/// A wrapper that contains the person data to populate a newly created source.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ContactToCreate {
    /// Required. The person data to populate a newly created source.
    #[serde(rename="contactPerson")]
    pub contact_person: Option<Person>,
}

impl client::Part for ContactToCreate {}


/// A request to copy an "Other contact" to my contacts group.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [copy other contact to my contacts group other contacts](OtherContactCopyOtherContactToMyContactsGroupCall) (request)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CopyOtherContactToMyContactsGroupRequest {
    /// Required. A field mask to restrict which fields are copied into the new contact. Valid values are: * emailAddresses * names * phoneNumbers
    #[serde(rename="copyMask")]
    pub copy_mask: Option<String>,
    /// Optional. A field mask to restrict which fields on the person are returned. Multiple fields can be specified by separating them with commas. Defaults to the copy mask with metadata and membership fields if not set. Valid values are: * addresses * ageRanges * biographies * birthdays * calendarUrls * clientData * coverPhotos * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * metadata * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * photos * relations * sipAddresses * skills * urls * userDefined
    #[serde(rename="readMask")]
    pub read_mask: Option<String>,
    /// Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT and READ_SOURCE_TYPE_PROFILE if not set.
    pub sources: Option<Vec<String>>,
}

impl client::RequestValue for CopyOtherContactToMyContactsGroupRequest {}


/// A person's cover photo. A large image shown on the person's profile page that represents who they are or what they care about.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CoverPhoto {
    /// True if the cover photo is the default cover photo; false if the cover photo is a user-provided cover photo.
    pub default: Option<bool>,
    /// Metadata about the cover photo.
    pub metadata: Option<FieldMetadata>,
    /// The URL of the cover photo.
    pub url: Option<String>,
}

impl client::Part for CoverPhoto {}


/// A request to create a new contact group.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [create contact groups](ContactGroupCreateCall) (request)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CreateContactGroupRequest {
    /// Required. The contact group to create.
    #[serde(rename="contactGroup")]
    pub contact_group: Option<ContactGroup>,
    /// Optional. A field mask to restrict which fields on the group are returned. Defaults to `metadata`, `groupType`, and `name` if not set or set to empty. Valid fields are: * clientData * groupType * metadata * name
    #[serde(rename="readGroupFields")]
    pub read_group_fields: Option<String>,
}

impl client::RequestValue for CreateContactGroupRequest {}


/// Represents a whole or partial calendar date, such as a birthday. The time of day and time zone are either specified elsewhere or are insignificant. The date is relative to the Gregorian Calendar. This can represent one of the following: * A full date, with non-zero year, month, and day values. * A month and day, with a zero year (for example, an anniversary). * A year on its own, with a zero month and a zero day. * A year and month, with a zero day (for example, a credit card expiration date). Related types: * google.type.TimeOfDay * google.type.DateTime * google.protobuf.Timestamp
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Date {
    /// Day of a month. Must be from 1 to 31 and valid for the year and month, or 0 to specify a year by itself or a year and month where the day isn't significant.
    pub day: Option<i32>,
    /// Month of a year. Must be from 1 to 12, or 0 to specify a year without a month and day.
    pub month: Option<i32>,
    /// Year of the date. Must be from 1 to 9999, or 0 to specify a date without a year.
    pub year: Option<i32>,
}

impl client::Part for Date {}


/// The response for deleting a contact's photo.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [delete contact photo people](PeopleDeleteContactPhotoCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct DeleteContactPhotoResponse {
    /// The updated person, if person_fields is set in the DeleteContactPhotoRequest; otherwise this will be unset.
    pub person: Option<Person>,
}

impl client::ResponseResult for DeleteContactPhotoResponse {}


/// A Google Workspace Domain membership.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct DomainMembership {
    /// True if the person is in the viewer's Google Workspace domain.
    #[serde(rename="inViewerDomain")]
    pub in_viewer_domain: Option<bool>,
}

impl client::Part for DomainMembership {}


/// A person's email address.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct EmailAddress {
    /// The display name of the email.
    #[serde(rename="displayName")]
    pub display_name: Option<String>,
    /// Output only. The type of the email address translated and formatted in the viewer's account locale or the `Accept-Language` HTTP header locale.
    #[serde(rename="formattedType")]
    pub formatted_type: Option<String>,
    /// Metadata about the email address.
    pub metadata: Option<FieldMetadata>,
    /// The type of the email address. The type can be custom or one of these predefined values: * `home` * `work` * `other`
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// The email address.
    pub value: Option<String>,
}

impl client::Part for EmailAddress {}


/// A generic empty message that you can re-use to avoid defining duplicated empty messages in your APIs. A typical example is to use it as the request or the response type of an API method. For instance: service Foo { rpc Bar(google.protobuf.Empty) returns (google.protobuf.Empty); } The JSON representation for `Empty` is empty JSON object `{}`.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [delete contact groups](ContactGroupDeleteCall) (response)
/// * [batch delete contacts people](PeopleBatchDeleteContactCall) (response)
/// * [delete contact people](PeopleDeleteContactCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Empty { _never_set: Option<bool> }

impl client::ResponseResult for Empty {}


/// An event related to the person.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    /// The date of the event.
    pub date: Option<Date>,
    /// Output only. The type of the event translated and formatted in the viewer's account locale or the `Accept-Language` HTTP header locale.
    #[serde(rename="formattedType")]
    pub formatted_type: Option<String>,
    /// Metadata about the event.
    pub metadata: Option<FieldMetadata>,
    /// The type of the event. The type can be custom or one of these predefined values: * `anniversary` * `other`
    #[serde(rename="type")]
    pub type_: Option<String>,
}

impl client::Part for Event {}


/// An identifier from an external entity related to the person.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ExternalId {
    /// Output only. The type of the event translated and formatted in the viewer's account locale or the `Accept-Language` HTTP header locale.
    #[serde(rename="formattedType")]
    pub formatted_type: Option<String>,
    /// Metadata about the external ID.
    pub metadata: Option<FieldMetadata>,
    /// The type of the external ID. The type can be custom or one of these predefined values: * `account` * `customer` * `loginId` * `network` * `organization`
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// The value of the external ID.
    pub value: Option<String>,
}

impl client::Part for ExternalId {}


/// Metadata about a field.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct FieldMetadata {
    /// Output only. True if the field is the primary field for all sources in the person. Each person will have at most one field with `primary` set to true.
    pub primary: Option<bool>,
    /// The source of the field.
    pub source: Option<Source>,
    /// True if the field is the primary field for the source. Each source must have at most one field with `source_primary` set to true.
    #[serde(rename="sourcePrimary")]
    pub source_primary: Option<bool>,
    /// Output only. True if the field is verified; false if the field is unverified. A verified field is typically a name, email address, phone number, or website that has been confirmed to be owned by the person.
    pub verified: Option<bool>,
}

impl client::Part for FieldMetadata {}


/// The name that should be used to sort the person in a list.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct FileAs {
    /// Metadata about the file-as.
    pub metadata: Option<FieldMetadata>,
    /// The file-as value
    pub value: Option<String>,
}

impl client::Part for FileAs {}


/// A person's gender.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Gender {
    /// Free form text field for pronouns that should be used to address the person. Common values are: * `he`/`him` * `she`/`her` * `they`/`them`
    #[serde(rename="addressMeAs")]
    pub address_me_as: Option<String>,
    /// Output only. The value of the gender translated and formatted in the viewer's account locale or the `Accept-Language` HTTP header locale. Unspecified or custom value are not localized.
    #[serde(rename="formattedValue")]
    pub formatted_value: Option<String>,
    /// Metadata about the gender.
    pub metadata: Option<FieldMetadata>,
    /// The gender for the person. The gender can be custom or one of these predefined values: * `male` * `female` * `unspecified`
    pub value: Option<String>,
}

impl client::Part for Gender {}


/// The response to a get request for a list of people by resource name.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [get batch get people](PeopleGetBatchGetCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct GetPeopleResponse {
    /// The response for each requested resource name.
    pub responses: Option<Vec<PersonResponse>>,
}

impl client::ResponseResult for GetPeopleResponse {}


/// Arbitrary client data that is populated by clients. Duplicate keys and values are allowed.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct GroupClientData {
    /// The client specified key of the client data.
    pub key: Option<String>,
    /// The client specified value of the client data.
    pub value: Option<String>,
}

impl client::Part for GroupClientData {}


/// A person's instant messaging client.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ImClient {
    /// Output only. The protocol of the IM client formatted in the viewer's account locale or the `Accept-Language` HTTP header locale.
    #[serde(rename="formattedProtocol")]
    pub formatted_protocol: Option<String>,
    /// Output only. The type of the IM client translated and formatted in the viewer's account locale or the `Accept-Language` HTTP header locale.
    #[serde(rename="formattedType")]
    pub formatted_type: Option<String>,
    /// Metadata about the IM client.
    pub metadata: Option<FieldMetadata>,
    /// The protocol of the IM client. The protocol can be custom or one of these predefined values: * `aim` * `msn` * `yahoo` * `skype` * `qq` * `googleTalk` * `icq` * `jabber` * `netMeeting`
    pub protocol: Option<String>,
    /// The type of the IM client. The type can be custom or one of these predefined values: * `home` * `work` * `other`
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// The user name used in the IM client.
    pub username: Option<String>,
}

impl client::Part for ImClient {}


/// One of the person's interests.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Interest {
    /// Metadata about the interest.
    pub metadata: Option<FieldMetadata>,
    /// The interest; for example, `stargazing`.
    pub value: Option<String>,
}

impl client::Part for Interest {}


/// The response to a request for the authenticated user's connections.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [connections list people](PeopleConnectionListCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ListConnectionsResponse {
    /// The list of people that the requestor is connected to.
    pub connections: Option<Vec<Person>>,
    /// A token, which can be sent as `page_token` to retrieve the next page. If this field is omitted, there are no subsequent pages.
    #[serde(rename="nextPageToken")]
    pub next_page_token: Option<String>,
    /// A token, which can be sent as `sync_token` to retrieve changes since the last request. Request must set `request_sync_token` to return the sync token. When the response is paginated, only the last page will contain `nextSyncToken`.
    #[serde(rename="nextSyncToken")]
    pub next_sync_token: Option<String>,
    /// The total number of items in the list without pagination.
    #[serde(rename="totalItems")]
    pub total_items: Option<i32>,
    /// **DEPRECATED** (Please use totalItems) The total number of people in the list without pagination.
    #[serde(rename="totalPeople")]
    pub total_people: Option<i32>,
}

impl client::ResponseResult for ListConnectionsResponse {}


/// The response to a list contact groups request.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [list contact groups](ContactGroupListCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ListContactGroupsResponse {
    /// The list of contact groups. Members of the contact groups are not populated.
    #[serde(rename="contactGroups")]
    pub contact_groups: Option<Vec<ContactGroup>>,
    /// The token that can be used to retrieve the next page of results.
    #[serde(rename="nextPageToken")]
    pub next_page_token: Option<String>,
    /// The token that can be used to retrieve changes since the last request.
    #[serde(rename="nextSyncToken")]
    pub next_sync_token: Option<String>,
    /// The total number of items in the list without pagination.
    #[serde(rename="totalItems")]
    pub total_items: Option<i32>,
}

impl client::ResponseResult for ListContactGroupsResponse {}


/// The response to a request for the authenticated user's domain directory.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [list directory people people](PeopleListDirectoryPeopleCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ListDirectoryPeopleResponse {
    /// A token, which can be sent as `page_token` to retrieve the next page. If this field is omitted, there are no subsequent pages.
    #[serde(rename="nextPageToken")]
    pub next_page_token: Option<String>,
    /// A token, which can be sent as `sync_token` to retrieve changes since the last request. Request must set `request_sync_token` to return the sync token.
    #[serde(rename="nextSyncToken")]
    pub next_sync_token: Option<String>,
    /// The list of people in the domain directory.
    pub people: Option<Vec<Person>>,
}

impl client::ResponseResult for ListDirectoryPeopleResponse {}


/// The response to a request for the authenticated user's "Other contacts".
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [list other contacts](OtherContactListCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ListOtherContactsResponse {
    /// A token, which can be sent as `page_token` to retrieve the next page. If this field is omitted, there are no subsequent pages.
    #[serde(rename="nextPageToken")]
    pub next_page_token: Option<String>,
    /// A token, which can be sent as `sync_token` to retrieve changes since the last request. Request must set `request_sync_token` to return the sync token.
    #[serde(rename="nextSyncToken")]
    pub next_sync_token: Option<String>,
    /// The list of "Other contacts" returned as Person resources. "Other contacts" support a limited subset of fields. See ListOtherContactsRequest.request_mask for more detailed information.
    #[serde(rename="otherContacts")]
    pub other_contacts: Option<Vec<Person>>,
    /// The total number of other contacts in the list without pagination.
    #[serde(rename="totalSize")]
    pub total_size: Option<i32>,
}

impl client::ResponseResult for ListOtherContactsResponse {}


/// A person's locale preference.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Locale {
    /// Metadata about the locale.
    pub metadata: Option<FieldMetadata>,
    /// The well-formed [IETF BCP 47](https://tools.ietf.org/html/bcp47) language tag representing the locale.
    pub value: Option<String>,
}

impl client::Part for Locale {}


/// A person's location.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Location {
    /// The building identifier.
    #[serde(rename="buildingId")]
    pub building_id: Option<String>,
    /// Whether the location is the current location.
    pub current: Option<bool>,
    /// The individual desk location.
    #[serde(rename="deskCode")]
    pub desk_code: Option<String>,
    /// The floor name or number.
    pub floor: Option<String>,
    /// The floor section in `floor_name`.
    #[serde(rename="floorSection")]
    pub floor_section: Option<String>,
    /// Metadata about the location.
    pub metadata: Option<FieldMetadata>,
    /// The type of the location. The type can be custom or one of these predefined values: * `desk` * `grewUp`
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// The free-form value of the location.
    pub value: Option<String>,
}

impl client::Part for Location {}


/// A person's membership in a group. Only contact group memberships can be modified.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Membership {
    /// The contact group membership.
    #[serde(rename="contactGroupMembership")]
    pub contact_group_membership: Option<ContactGroupMembership>,
    /// Output only. The domain membership.
    #[serde(rename="domainMembership")]
    pub domain_membership: Option<DomainMembership>,
    /// Metadata about the membership.
    pub metadata: Option<FieldMetadata>,
}

impl client::Part for Membership {}


/// A person's miscellaneous keyword.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct MiscKeyword {
    /// Output only. The type of the miscellaneous keyword translated and formatted in the viewer's account locale or the `Accept-Language` HTTP header locale.
    #[serde(rename="formattedType")]
    pub formatted_type: Option<String>,
    /// Metadata about the miscellaneous keyword.
    pub metadata: Option<FieldMetadata>,
    /// The miscellaneous keyword type.
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// The value of the miscellaneous keyword.
    pub value: Option<String>,
}

impl client::Part for MiscKeyword {}


/// A request to modify an existing contact group's members. Contacts can be removed from any group but they can only be added to a user group or "myContacts" or "starred" system groups.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [members modify contact groups](ContactGroupMemberModifyCall) (request)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ModifyContactGroupMembersRequest {
    /// Optional. The resource names of the contact people to add in the form of `people/{person_id}`. The total number of resource names in `resource_names_to_add` and `resource_names_to_remove` must be less than or equal to 1000.
    #[serde(rename="resourceNamesToAdd")]
    pub resource_names_to_add: Option<Vec<String>>,
    /// Optional. The resource names of the contact people to remove in the form of `people/{person_id}`. The total number of resource names in `resource_names_to_add` and `resource_names_to_remove` must be less than or equal to 1000.
    #[serde(rename="resourceNamesToRemove")]
    pub resource_names_to_remove: Option<Vec<String>>,
}

impl client::RequestValue for ModifyContactGroupMembersRequest {}


/// The response to a modify contact group members request.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [members modify contact groups](ContactGroupMemberModifyCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ModifyContactGroupMembersResponse {
    /// The contact people resource names that cannot be removed from their last contact group.
    #[serde(rename="canNotRemoveLastContactGroupResourceNames")]
    pub can_not_remove_last_contact_group_resource_names: Option<Vec<String>>,
    /// The contact people resource names that were not found.
    #[serde(rename="notFoundResourceNames")]
    pub not_found_resource_names: Option<Vec<String>>,
}

impl client::ResponseResult for ModifyContactGroupMembersResponse {}


/// A person's name. If the name is a mononym, the family name is empty.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Name {
    /// Output only. The display name formatted according to the locale specified by the viewer's account or the `Accept-Language` HTTP header.
    #[serde(rename="displayName")]
    pub display_name: Option<String>,
    /// Output only. The display name with the last name first formatted according to the locale specified by the viewer's account or the `Accept-Language` HTTP header.
    #[serde(rename="displayNameLastFirst")]
    pub display_name_last_first: Option<String>,
    /// The family name.
    #[serde(rename="familyName")]
    pub family_name: Option<String>,
    /// The given name.
    #[serde(rename="givenName")]
    pub given_name: Option<String>,
    /// The honorific prefixes, such as `Mrs.` or `Dr.`
    #[serde(rename="honorificPrefix")]
    pub honorific_prefix: Option<String>,
    /// The honorific suffixes, such as `Jr.`
    #[serde(rename="honorificSuffix")]
    pub honorific_suffix: Option<String>,
    /// Metadata about the name.
    pub metadata: Option<FieldMetadata>,
    /// The middle name(s).
    #[serde(rename="middleName")]
    pub middle_name: Option<String>,
    /// The family name spelled as it sounds.
    #[serde(rename="phoneticFamilyName")]
    pub phonetic_family_name: Option<String>,
    /// The full name spelled as it sounds.
    #[serde(rename="phoneticFullName")]
    pub phonetic_full_name: Option<String>,
    /// The given name spelled as it sounds.
    #[serde(rename="phoneticGivenName")]
    pub phonetic_given_name: Option<String>,
    /// The honorific prefixes spelled as they sound.
    #[serde(rename="phoneticHonorificPrefix")]
    pub phonetic_honorific_prefix: Option<String>,
    /// The honorific suffixes spelled as they sound.
    #[serde(rename="phoneticHonorificSuffix")]
    pub phonetic_honorific_suffix: Option<String>,
    /// The middle name(s) spelled as they sound.
    #[serde(rename="phoneticMiddleName")]
    pub phonetic_middle_name: Option<String>,
    /// The free form name value.
    #[serde(rename="unstructuredName")]
    pub unstructured_name: Option<String>,
}

impl client::Part for Name {}


/// A person's nickname.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Nickname {
    /// Metadata about the nickname.
    pub metadata: Option<FieldMetadata>,
    /// The type of the nickname.
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// The nickname.
    pub value: Option<String>,
}

impl client::Part for Nickname {}


/// A person's occupation.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Occupation {
    /// Metadata about the occupation.
    pub metadata: Option<FieldMetadata>,
    /// The occupation; for example, `carpenter`.
    pub value: Option<String>,
}

impl client::Part for Occupation {}


/// A person's past or current organization. Overlapping date ranges are permitted.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Organization {
    /// The person's cost center at the organization.
    #[serde(rename="costCenter")]
    pub cost_center: Option<String>,
    /// True if the organization is the person's current organization; false if the organization is a past organization.
    pub current: Option<bool>,
    /// The person's department at the organization.
    pub department: Option<String>,
    /// The domain name associated with the organization; for example, `google.com`.
    pub domain: Option<String>,
    /// The end date when the person left the organization.
    #[serde(rename="endDate")]
    pub end_date: Option<Date>,
    /// Output only. The type of the organization translated and formatted in the viewer's account locale or the `Accept-Language` HTTP header locale.
    #[serde(rename="formattedType")]
    pub formatted_type: Option<String>,
    /// The person's full-time equivalent millipercent within the organization (100000 = 100%).
    #[serde(rename="fullTimeEquivalentMillipercent")]
    pub full_time_equivalent_millipercent: Option<i32>,
    /// The person's job description at the organization.
    #[serde(rename="jobDescription")]
    pub job_description: Option<String>,
    /// The location of the organization office the person works at.
    pub location: Option<String>,
    /// Metadata about the organization.
    pub metadata: Option<FieldMetadata>,
    /// The name of the organization.
    pub name: Option<String>,
    /// The phonetic name of the organization.
    #[serde(rename="phoneticName")]
    pub phonetic_name: Option<String>,
    /// The start date when the person joined the organization.
    #[serde(rename="startDate")]
    pub start_date: Option<Date>,
    /// The symbol associated with the organization; for example, a stock ticker symbol, abbreviation, or acronym.
    pub symbol: Option<String>,
    /// The person's job title at the organization.
    pub title: Option<String>,
    /// The type of the organization. The type can be custom or one of these predefined values: * `work` * `school`
    #[serde(rename="type")]
    pub type_: Option<String>,
}

impl client::Part for Organization {}


/// Information about a person merged from various data sources such as the authenticated user's contacts and profile data. Most fields can have multiple items. The items in a field have no guaranteed order, but each non-empty field is guaranteed to have exactly one field with `metadata.primary` set to true.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [copy other contact to my contacts group other contacts](OtherContactCopyOtherContactToMyContactsGroupCall) (response)
/// * [create contact people](PeopleCreateContactCall) (request|response)
/// * [get people](PeopleGetCall) (response)
/// * [update contact people](PeopleUpdateContactCall) (request|response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    /// The person's street addresses.
    pub addresses: Option<Vec<Address>>,
    /// Output only. **DEPRECATED** (Please use `person.ageRanges` instead) The person's age range.
    #[serde(rename="ageRange")]
    pub age_range: Option<String>,
    /// Output only. The person's age ranges.
    #[serde(rename="ageRanges")]
    pub age_ranges: Option<Vec<AgeRangeType>>,
    /// The person's biographies. This field is a singleton for contact sources.
    pub biographies: Option<Vec<Biography>>,
    /// The person's birthdays. This field is a singleton for contact sources.
    pub birthdays: Option<Vec<Birthday>>,
    /// **DEPRECATED**: No data will be returned The person's bragging rights.
    #[serde(rename="braggingRights")]
    pub bragging_rights: Option<Vec<BraggingRights>>,
    /// The person's calendar URLs.
    #[serde(rename="calendarUrls")]
    pub calendar_urls: Option<Vec<CalendarUrl>>,
    /// The person's client data.
    #[serde(rename="clientData")]
    pub client_data: Option<Vec<ClientData>>,
    /// Output only. The person's cover photos.
    #[serde(rename="coverPhotos")]
    pub cover_photos: Option<Vec<CoverPhoto>>,
    /// The person's email addresses. For `people.connections.list` and `otherContacts.list` the number of email addresses is limited to 100. If a Person has more email addresses the entire set can be obtained by calling GetPeople.
    #[serde(rename="emailAddresses")]
    pub email_addresses: Option<Vec<EmailAddress>>,
    /// The [HTTP entity tag](https://en.wikipedia.org/wiki/HTTP_ETag) of the resource. Used for web cache validation.
    pub etag: Option<String>,
    /// The person's events.
    pub events: Option<Vec<Event>>,
    /// The person's external IDs.
    #[serde(rename="externalIds")]
    pub external_ids: Option<Vec<ExternalId>>,
    /// The person's file-ases.
    #[serde(rename="fileAses")]
    pub file_ases: Option<Vec<FileAs>>,
    /// The person's genders. This field is a singleton for contact sources.
    pub genders: Option<Vec<Gender>>,
    /// The person's instant messaging clients.
    #[serde(rename="imClients")]
    pub im_clients: Option<Vec<ImClient>>,
    /// The person's interests.
    pub interests: Option<Vec<Interest>>,
    /// The person's locale preferences.
    pub locales: Option<Vec<Locale>>,
    /// The person's locations.
    pub locations: Option<Vec<Location>>,
    /// The person's group memberships.
    pub memberships: Option<Vec<Membership>>,
    /// Output only. Metadata about the person.
    pub metadata: Option<PersonMetadata>,
    /// The person's miscellaneous keywords.
    #[serde(rename="miscKeywords")]
    pub misc_keywords: Option<Vec<MiscKeyword>>,
    /// The person's names. This field is a singleton for contact sources.
    pub names: Option<Vec<Name>>,
    /// The person's nicknames.
    pub nicknames: Option<Vec<Nickname>>,
    /// The person's occupations.
    pub occupations: Option<Vec<Occupation>>,
    /// The person's past or current organizations.
    pub organizations: Option<Vec<Organization>>,
    /// The person's phone numbers. For `people.connections.list` and `otherContacts.list` the number of phone numbers is limited to 100. If a Person has more phone numbers the entire set can be obtained by calling GetPeople.
    #[serde(rename="phoneNumbers")]
    pub phone_numbers: Option<Vec<PhoneNumber>>,
    /// Output only. The person's photos.
    pub photos: Option<Vec<Photo>>,
    /// The person's relations.
    pub relations: Option<Vec<Relation>>,
    /// Output only. **DEPRECATED**: No data will be returned The person's relationship interests.
    #[serde(rename="relationshipInterests")]
    pub relationship_interests: Option<Vec<RelationshipInterest>>,
    /// Output only. **DEPRECATED**: No data will be returned The person's relationship statuses.
    #[serde(rename="relationshipStatuses")]
    pub relationship_statuses: Option<Vec<RelationshipStatus>>,
    /// **DEPRECATED**: (Please use `person.locations` instead) The person's residences.
    pub residences: Option<Vec<Residence>>,
    /// The resource name for the person, assigned by the server. An ASCII string with a max length of 27 characters, in the form of `people/{person_id}`.
    #[serde(rename="resourceName")]
    pub resource_name: Option<String>,
    /// The person's SIP addresses.
    #[serde(rename="sipAddresses")]
    pub sip_addresses: Option<Vec<SipAddress>>,
    /// The person's skills.
    pub skills: Option<Vec<Skill>>,
    /// Output only. **DEPRECATED**: No data will be returned The person's taglines.
    pub taglines: Option<Vec<Tagline>>,
    /// The person's associated URLs.
    pub urls: Option<Vec<Url>>,
    /// The person's user defined data.
    #[serde(rename="userDefined")]
    pub user_defined: Option<Vec<UserDefined>>,
}

impl client::RequestValue for Person {}
impl client::ResponseResult for Person {}


/// The metadata about a person.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PersonMetadata {
    /// Output only. True if the person resource has been deleted. Populated only for `people.connections.list` and `otherContacts.list` sync requests.
    pub deleted: Option<bool>,
    /// Output only. Resource names of people linked to this resource.
    #[serde(rename="linkedPeopleResourceNames")]
    pub linked_people_resource_names: Option<Vec<String>>,
    /// Output only. **DEPRECATED** (Please use `person.metadata.sources.profileMetadata.objectType` instead) The type of the person object.
    #[serde(rename="objectType")]
    pub object_type: Option<String>,
    /// Output only. Any former resource names this person has had. Populated only for `people.connections.list` requests that include a sync token. The resource name may change when adding or removing fields that link a contact and profile such as a verified email, verified phone number, or profile URL.
    #[serde(rename="previousResourceNames")]
    pub previous_resource_names: Option<Vec<String>>,
    /// The sources of data for the person.
    pub sources: Option<Vec<Source>>,
}

impl client::Part for PersonMetadata {}


/// The response for a single person
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PersonResponse {
    /// **DEPRECATED** (Please use status instead) [HTTP 1.1 status code] (http://www.w3.org/Protocols/rfc2616/rfc2616-sec10.html).
    #[serde(rename="httpStatusCode")]
    pub http_status_code: Option<i32>,
    /// The person.
    pub person: Option<Person>,
    /// The original requested resource name. May be different than the resource name on the returned person. The resource name can change when adding or removing fields that link a contact and profile such as a verified email, verified phone number, or a profile URL.
    #[serde(rename="requestedResourceName")]
    pub requested_resource_name: Option<String>,
    /// The status of the response.
    pub status: Option<Status>,
}

impl client::Part for PersonResponse {}


/// A person's phone number.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PhoneNumber {
    /// Output only. The canonicalized [ITU-T E.164](https://law.resource.org/pub/us/cfr/ibr/004/itu-t.E.164.1.2008.pdf) form of the phone number.
    #[serde(rename="canonicalForm")]
    pub canonical_form: Option<String>,
    /// Output only. The type of the phone number translated and formatted in the viewer's account locale or the `Accept-Language` HTTP header locale.
    #[serde(rename="formattedType")]
    pub formatted_type: Option<String>,
    /// Metadata about the phone number.
    pub metadata: Option<FieldMetadata>,
    /// The type of the phone number. The type can be custom or one of these predefined values: * `home` * `work` * `mobile` * `homeFax` * `workFax` * `otherFax` * `pager` * `workMobile` * `workPager` * `main` * `googleVoice` * `other`
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// The phone number.
    pub value: Option<String>,
}

impl client::Part for PhoneNumber {}


/// A person's photo. A picture shown next to the person's name to help others recognize the person.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Photo {
    /// True if the photo is a default photo; false if the photo is a user-provided photo.
    pub default: Option<bool>,
    /// Metadata about the photo.
    pub metadata: Option<FieldMetadata>,
    /// The URL of the photo. You can change the desired size by appending a query parameter `sz={size}` at the end of the url, where {size} is the size in pixels. Example: https://lh3.googleusercontent.com/-T_wVWLlmg7w/AAAAAAAAAAI/AAAAAAAABa8/00gzXvDBYqw/s100/photo.jpg?sz=50
    pub url: Option<String>,
}

impl client::Part for Photo {}


/// The metadata about a profile.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ProfileMetadata {
    /// Output only. The profile object type.
    #[serde(rename="objectType")]
    pub object_type: Option<String>,
    /// Output only. The user types.
    #[serde(rename="userTypes")]
    pub user_types: Option<Vec<String>>,
}

impl client::Part for ProfileMetadata {}


/// A person's relation to another person.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Relation {
    /// Output only. The type of the relation translated and formatted in the viewer's account locale or the locale specified in the Accept-Language HTTP header.
    #[serde(rename="formattedType")]
    pub formatted_type: Option<String>,
    /// Metadata about the relation.
    pub metadata: Option<FieldMetadata>,
    /// The name of the other person this relation refers to.
    pub person: Option<String>,
    /// The person's relation to the other person. The type can be custom or one of these predefined values: * `spouse` * `child` * `mother` * `father` * `parent` * `brother` * `sister` * `friend` * `relative` * `domesticPartner` * `manager` * `assistant` * `referredBy` * `partner`
    #[serde(rename="type")]
    pub type_: Option<String>,
}

impl client::Part for Relation {}


/// **DEPRECATED**: No data will be returned A person's relationship interest .
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipInterest {
    /// Output only. The value of the relationship interest translated and formatted in the viewer's account locale or the locale specified in the Accept-Language HTTP header.
    #[serde(rename="formattedValue")]
    pub formatted_value: Option<String>,
    /// Metadata about the relationship interest.
    pub metadata: Option<FieldMetadata>,
    /// The kind of relationship the person is looking for. The value can be custom or one of these predefined values: * `friend` * `date` * `relationship` * `networking`
    pub value: Option<String>,
}

impl client::Part for RelationshipInterest {}


/// **DEPRECATED**: No data will be returned A person's relationship status.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipStatus {
    /// Output only. The value of the relationship status translated and formatted in the viewer's account locale or the `Accept-Language` HTTP header locale.
    #[serde(rename="formattedValue")]
    pub formatted_value: Option<String>,
    /// Metadata about the relationship status.
    pub metadata: Option<FieldMetadata>,
    /// The relationship status. The value can be custom or one of these predefined values: * `single` * `inARelationship` * `engaged` * `married` * `itsComplicated` * `openRelationship` * `widowed` * `inDomesticPartnership` * `inCivilUnion`
    pub value: Option<String>,
}

impl client::Part for RelationshipStatus {}


/// **DEPRECATED**: Please use `person.locations` instead. A person's past or current residence.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Residence {
    /// True if the residence is the person's current residence; false if the residence is a past residence.
    pub current: Option<bool>,
    /// Metadata about the residence.
    pub metadata: Option<FieldMetadata>,
    /// The address of the residence.
    pub value: Option<String>,
}

impl client::Part for Residence {}


/// The response to a request for people in the authenticated user's domain directory that match the specified query.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [search directory people people](PeopleSearchDirectoryPeopleCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct SearchDirectoryPeopleResponse {
    /// A token, which can be sent as `page_token` to retrieve the next page. If this field is omitted, there are no subsequent pages.
    #[serde(rename="nextPageToken")]
    pub next_page_token: Option<String>,
    /// The list of people in the domain directory that match the query.
    pub people: Option<Vec<Person>>,
    /// The total number of items in the list without pagination.
    #[serde(rename="totalSize")]
    pub total_size: Option<i32>,
}

impl client::ResponseResult for SearchDirectoryPeopleResponse {}


/// The response to a search request for the authenticated user, given a query.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [search other contacts](OtherContactSearchCall) (response)
/// * [search contacts people](PeopleSearchContactCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    /// The results of the request.
    pub results: Option<Vec<SearchResult>>,
}

impl client::ResponseResult for SearchResponse {}


/// A result of a search query.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matched Person.
    pub person: Option<Person>,
}

impl client::Part for SearchResult {}


/// A person's SIP address. Session Initial Protocol addresses are used for VoIP communications to make voice or video calls over the internet.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct SipAddress {
    /// Output only. The type of the SIP address translated and formatted in the viewer's account locale or the `Accept-Language` HTTP header locale.
    #[serde(rename="formattedType")]
    pub formatted_type: Option<String>,
    /// Metadata about the SIP address.
    pub metadata: Option<FieldMetadata>,
    /// The type of the SIP address. The type can be custom or or one of these predefined values: * `home` * `work` * `mobile` * `other`
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// The SIP address in the [RFC 3261 19.1](https://tools.ietf.org/html/rfc3261#section-19.1) SIP URI format.
    pub value: Option<String>,
}

impl client::Part for SipAddress {}


/// A skill that the person has.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Skill {
    /// Metadata about the skill.
    pub metadata: Option<FieldMetadata>,
    /// The skill; for example, `underwater basket weaving`.
    pub value: Option<String>,
}

impl client::Part for Skill {}


/// The source of a field.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    /// **Only populated in `person.metadata.sources`.** The [HTTP entity tag](https://en.wikipedia.org/wiki/HTTP_ETag) of the source. Used for web cache validation.
    pub etag: Option<String>,
    /// The unique identifier within the source type generated by the server.
    pub id: Option<String>,
    /// Output only. **Only populated in `person.metadata.sources`.** Metadata about a source of type PROFILE.
    #[serde(rename="profileMetadata")]
    pub profile_metadata: Option<ProfileMetadata>,
    /// The source type.
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// Output only. **Only populated in `person.metadata.sources`.** Last update timestamp of this source.
    #[serde(rename="updateTime")]
    pub update_time: Option<String>,
}

impl client::Part for Source {}


/// The `Status` type defines a logical error model that is suitable for different programming environments, including REST APIs and RPC APIs. It is used by [gRPC](https://github.com/grpc). Each `Status` message contains three pieces of data: error code, error message, and error details. You can find out more about this error model and how to work with it in the [API Design Guide](https://cloud.google.com/apis/design/errors).
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Status {
    /// The status code, which should be an enum value of google.rpc.Code.
    pub code: Option<i32>,
    /// A list of messages that carry the error details. There is a common set of message types for APIs to use.
    pub details: Option<Vec<HashMap<String, String>>>,
    /// A developer-facing error message, which should be in English. Any user-facing error message should be localized and sent in the google.rpc.Status.details field, or localized by the client.
    pub message: Option<String>,
}

impl client::Part for Status {}


/// **DEPRECATED**: No data will be returned A brief one-line description of the person.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Tagline {
    /// Metadata about the tagline.
    pub metadata: Option<FieldMetadata>,
    /// The tagline.
    pub value: Option<String>,
}

impl client::Part for Tagline {}


/// A request to update an existing user contact group. All updated fields will be replaced.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [update contact groups](ContactGroupUpdateCall) (request)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct UpdateContactGroupRequest {
    /// Required. The contact group to update.
    #[serde(rename="contactGroup")]
    pub contact_group: Option<ContactGroup>,
    /// Optional. A field mask to restrict which fields on the group are returned. Defaults to `metadata`, `groupType`, and `name` if not set or set to empty. Valid fields are: * clientData * groupType * memberCount * metadata * name
    #[serde(rename="readGroupFields")]
    pub read_group_fields: Option<String>,
    /// Optional. A field mask to restrict which fields on the group are updated. Multiple fields can be specified by separating them with commas. Defaults to `name` if not set or set to empty. Updated fields are replaced. Valid values are: * clientData * name
    #[serde(rename="updateGroupFields")]
    pub update_group_fields: Option<String>,
}

impl client::RequestValue for UpdateContactGroupRequest {}


/// A request to update an existing contact's photo. All requests must have a valid photo format: JPEG or PNG.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [update contact photo people](PeopleUpdateContactPhotoCall) (request)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct UpdateContactPhotoRequest {
    /// Optional. A field mask to restrict which fields on the person are returned. Multiple fields can be specified by separating them with commas. Defaults to empty if not set, which will skip the post mutate get. Valid values are: * addresses * ageRanges * biographies * birthdays * calendarUrls * clientData * coverPhotos * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * metadata * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * photos * relations * sipAddresses * skills * urls * userDefined
    #[serde(rename="personFields")]
    pub person_fields: Option<String>,
    /// Required. Raw photo bytes
    #[serde(rename="photoBytes")]
    pub photo_bytes: Option<String>,
    /// Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT and READ_SOURCE_TYPE_PROFILE if not set.
    pub sources: Option<Vec<String>>,
}

impl client::RequestValue for UpdateContactPhotoRequest {}


/// The response for updating a contact's photo.
/// 
/// # Activities
/// 
/// This type is used in activities, which are methods you may call on this type or where this type is involved in. 
/// The list links the activity name, along with information about where it is used (one of *request* and *response*).
/// 
/// * [update contact photo people](PeopleUpdateContactPhotoCall) (response)
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct UpdateContactPhotoResponse {
    /// The updated person, if person_fields is set in the UpdateContactPhotoRequest; otherwise this will be unset.
    pub person: Option<Person>,
}

impl client::ResponseResult for UpdateContactPhotoResponse {}


/// A person's associated URLs.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Url {
    /// Output only. The type of the URL translated and formatted in the viewer's account locale or the `Accept-Language` HTTP header locale.
    #[serde(rename="formattedType")]
    pub formatted_type: Option<String>,
    /// Metadata about the URL.
    pub metadata: Option<FieldMetadata>,
    /// The type of the URL. The type can be custom or one of these predefined values: * `home` * `work` * `blog` * `profile` * `homePage` * `ftp` * `reservations` * `appInstallPage`: website for a Currents application. * `other`
    #[serde(rename="type")]
    pub type_: Option<String>,
    /// The URL.
    pub value: Option<String>,
}

impl client::Part for Url {}


/// Arbitrary user data that is populated by the end users.
/// 
/// This type is not used in any activity, and only used as *part* of another schema.
/// 
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct UserDefined {
    /// The end user specified key of the user defined data.
    pub key: Option<String>,
    /// Metadata about the user defined data.
    pub metadata: Option<FieldMetadata>,
    /// The end user specified value of the user defined data.
    pub value: Option<String>,
}

impl client::Part for UserDefined {}



// ###################
// MethodBuilders ###
// #################

/// A builder providing access to all methods supported on *contactGroup* resources.
/// It is not used directly, but through the `PeopleService` hub.
///
/// # Example
///
/// Instantiate a resource builder
///
/// ```test_harness,no_run
/// extern crate hyper;
/// extern crate hyper_rustls;
/// extern crate google_people1 as people1;
/// 
/// # async fn dox() {
/// use std::default::Default;
/// use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// let secret: oauth2::ApplicationSecret = Default::default();
/// let auth = oauth2::InstalledFlowAuthenticator::builder(
///         secret,
///         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
///     ).build().await.unwrap();
/// let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // Usually you wouldn't bind this to a variable, but keep calling *CallBuilders*
/// // like `batch_get(...)`, `create(...)`, `delete(...)`, `get(...)`, `list(...)`, `members_modify(...)` and `update(...)`
/// // to build up your call.
/// let rb = hub.contact_groups();
/// # }
/// ```
pub struct ContactGroupMethods<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
}

impl<'a, S> client::MethodsBuilder for ContactGroupMethods<'a, S> {}

impl<'a, S> ContactGroupMethods<'a, S> {
    
    /// Create a builder to help you perform the following task:
    ///
    /// Modify the members of a contact group owned by the authenticated user. The only system contact groups that can have members added are `contactGroups/myContacts` and `contactGroups/starred`. Other system contact groups are deprecated and can only have contacts removed.
    /// 
    /// # Arguments
    ///
    /// * `request` - No description provided.
    /// * `resourceName` - Required. The resource name of the contact group to modify.
    pub fn members_modify(&self, request: ModifyContactGroupMembersRequest, resource_name: &str) -> ContactGroupMemberModifyCall<'a, S> {
        ContactGroupMemberModifyCall {
            hub: self.hub,
            _request: request,
            _resource_name: resource_name.to_string(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Get a list of contact groups owned by the authenticated user by specifying a list of contact group resource names.
    pub fn batch_get(&self) -> ContactGroupBatchGetCall<'a, S> {
        ContactGroupBatchGetCall {
            hub: self.hub,
            _resource_names: Default::default(),
            _max_members: Default::default(),
            _group_fields: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Create a new contact group owned by the authenticated user. Created contact group names must be unique to the users contact groups. Attempting to create a group with a duplicate name will return a HTTP 409 error. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
    /// 
    /// # Arguments
    ///
    /// * `request` - No description provided.
    pub fn create(&self, request: CreateContactGroupRequest) -> ContactGroupCreateCall<'a, S> {
        ContactGroupCreateCall {
            hub: self.hub,
            _request: request,
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Delete an existing contact group owned by the authenticated user by specifying a contact group resource name. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
    /// 
    /// # Arguments
    ///
    /// * `resourceName` - Required. The resource name of the contact group to delete.
    pub fn delete(&self, resource_name: &str) -> ContactGroupDeleteCall<'a, S> {
        ContactGroupDeleteCall {
            hub: self.hub,
            _resource_name: resource_name.to_string(),
            _delete_contacts: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Get a specific contact group owned by the authenticated user by specifying a contact group resource name.
    /// 
    /// # Arguments
    ///
    /// * `resourceName` - Required. The resource name of the contact group to get.
    pub fn get(&self, resource_name: &str) -> ContactGroupGetCall<'a, S> {
        ContactGroupGetCall {
            hub: self.hub,
            _resource_name: resource_name.to_string(),
            _max_members: Default::default(),
            _group_fields: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// List all contact groups owned by the authenticated user. Members of the contact groups are not populated.
    pub fn list(&self) -> ContactGroupListCall<'a, S> {
        ContactGroupListCall {
            hub: self.hub,
            _sync_token: Default::default(),
            _page_token: Default::default(),
            _page_size: Default::default(),
            _group_fields: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Update the name of an existing contact group owned by the authenticated user. Updated contact group names must be unique to the users contact groups. Attempting to create a group with a duplicate name will return a HTTP 409 error. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
    /// 
    /// # Arguments
    ///
    /// * `request` - No description provided.
    /// * `resourceName` - The resource name for the contact group, assigned by the server. An ASCII string, in the form of `contactGroups/{contact_group_id}`.
    pub fn update(&self, request: UpdateContactGroupRequest, resource_name: &str) -> ContactGroupUpdateCall<'a, S> {
        ContactGroupUpdateCall {
            hub: self.hub,
            _request: request,
            _resource_name: resource_name.to_string(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
}



/// A builder providing access to all methods supported on *otherContact* resources.
/// It is not used directly, but through the `PeopleService` hub.
///
/// # Example
///
/// Instantiate a resource builder
///
/// ```test_harness,no_run
/// extern crate hyper;
/// extern crate hyper_rustls;
/// extern crate google_people1 as people1;
/// 
/// # async fn dox() {
/// use std::default::Default;
/// use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// let secret: oauth2::ApplicationSecret = Default::default();
/// let auth = oauth2::InstalledFlowAuthenticator::builder(
///         secret,
///         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
///     ).build().await.unwrap();
/// let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // Usually you wouldn't bind this to a variable, but keep calling *CallBuilders*
/// // like `copy_other_contact_to_my_contacts_group(...)`, `list(...)` and `search(...)`
/// // to build up your call.
/// let rb = hub.other_contacts();
/// # }
/// ```
pub struct OtherContactMethods<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
}

impl<'a, S> client::MethodsBuilder for OtherContactMethods<'a, S> {}

impl<'a, S> OtherContactMethods<'a, S> {
    
    /// Create a builder to help you perform the following task:
    ///
    /// Copies an "Other contact" to a new contact in the user's "myContacts" group Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
    /// 
    /// # Arguments
    ///
    /// * `request` - No description provided.
    /// * `resourceName` - Required. The resource name of the "Other contact" to copy.
    pub fn copy_other_contact_to_my_contacts_group(&self, request: CopyOtherContactToMyContactsGroupRequest, resource_name: &str) -> OtherContactCopyOtherContactToMyContactsGroupCall<'a, S> {
        OtherContactCopyOtherContactToMyContactsGroupCall {
            hub: self.hub,
            _request: request,
            _resource_name: resource_name.to_string(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// List all "Other contacts", that is contacts that are not in a contact group. "Other contacts" are typically auto created contacts from interactions. Sync tokens expire 7 days after the full sync. A request with an expired sync token will result in a 410 error. In the case of such an error clients should make a full sync request without a `sync_token`. The first page of a full sync request has an additional quota. If the quota is exceeded, a 429 error will be returned. This quota is fixed and can not be increased. When the `sync_token` is specified, resources deleted since the last sync will be returned as a person with `PersonMetadata.deleted` set to true. When the `page_token` or `sync_token` is specified, all other request parameters must match the first call. Writes may have a propagation delay of several minutes for sync requests. Incremental syncs are not intended for read-after-write use cases. See example usage at [List the user's other contacts that have changed](/people/v1/other-contacts#list_the_users_other_contacts_that_have_changed).
    pub fn list(&self) -> OtherContactListCall<'a, S> {
        OtherContactListCall {
            hub: self.hub,
            _sync_token: Default::default(),
            _sources: Default::default(),
            _request_sync_token: Default::default(),
            _read_mask: Default::default(),
            _page_token: Default::default(),
            _page_size: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Provides a list of contacts in the authenticated user's other contacts that matches the search query. The query matches on a contact's `names`, `emailAddresses`, and `phoneNumbers` fields that are from the OTHER_CONTACT source. **IMPORTANT**: Before searching, clients should send a warmup request with an empty query to update the cache. See https://developers.google.com/people/v1/other-contacts#search_the_users_other_contacts
    pub fn search(&self) -> OtherContactSearchCall<'a, S> {
        OtherContactSearchCall {
            hub: self.hub,
            _read_mask: Default::default(),
            _query: Default::default(),
            _page_size: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
}



/// A builder providing access to all methods supported on *people* resources.
/// It is not used directly, but through the `PeopleService` hub.
///
/// # Example
///
/// Instantiate a resource builder
///
/// ```test_harness,no_run
/// extern crate hyper;
/// extern crate hyper_rustls;
/// extern crate google_people1 as people1;
/// 
/// # async fn dox() {
/// use std::default::Default;
/// use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// let secret: oauth2::ApplicationSecret = Default::default();
/// let auth = oauth2::InstalledFlowAuthenticator::builder(
///         secret,
///         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
///     ).build().await.unwrap();
/// let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // Usually you wouldn't bind this to a variable, but keep calling *CallBuilders*
/// // like `batch_create_contacts(...)`, `batch_delete_contacts(...)`, `batch_update_contacts(...)`, `connections_list(...)`, `create_contact(...)`, `delete_contact(...)`, `delete_contact_photo(...)`, `get(...)`, `get_batch_get(...)`, `list_directory_people(...)`, `search_contacts(...)`, `search_directory_people(...)`, `update_contact(...)` and `update_contact_photo(...)`
/// // to build up your call.
/// let rb = hub.people();
/// # }
/// ```
pub struct PeopleMethods<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
}

impl<'a, S> client::MethodsBuilder for PeopleMethods<'a, S> {}

impl<'a, S> PeopleMethods<'a, S> {
    
    /// Create a builder to help you perform the following task:
    ///
    /// Provides a list of the authenticated user's contacts. Sync tokens expire 7 days after the full sync. A request with an expired sync token will result in a 410 error. In the case of such an error clients should make a full sync request without a `sync_token`. The first page of a full sync request has an additional quota. If the quota is exceeded, a 429 error will be returned. This quota is fixed and can not be increased. When the `sync_token` is specified, resources deleted since the last sync will be returned as a person with `PersonMetadata.deleted` set to true. When the `page_token` or `sync_token` is specified, all other request parameters must match the first call. Writes may have a propagation delay of several minutes for sync requests. Incremental syncs are not intended for read-after-write use cases. See example usage at [List the user's contacts that have changed](/people/v1/contacts#list_the_users_contacts_that_have_changed).
    /// 
    /// # Arguments
    ///
    /// * `resourceName` - Required. The resource name to return connections for. Only `people/me` is valid.
    pub fn connections_list(&self, resource_name: &str) -> PeopleConnectionListCall<'a, S> {
        PeopleConnectionListCall {
            hub: self.hub,
            _resource_name: resource_name.to_string(),
            _sync_token: Default::default(),
            _sources: Default::default(),
            _sort_order: Default::default(),
            _request_sync_token: Default::default(),
            _request_mask_include_field: Default::default(),
            _person_fields: Default::default(),
            _page_token: Default::default(),
            _page_size: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Create a batch of new contacts and return the PersonResponses for the newly Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
    /// 
    /// # Arguments
    ///
    /// * `request` - No description provided.
    pub fn batch_create_contacts(&self, request: BatchCreateContactsRequest) -> PeopleBatchCreateContactCall<'a, S> {
        PeopleBatchCreateContactCall {
            hub: self.hub,
            _request: request,
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Delete a batch of contacts. Any non-contact data will not be deleted. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
    /// 
    /// # Arguments
    ///
    /// * `request` - No description provided.
    pub fn batch_delete_contacts(&self, request: BatchDeleteContactsRequest) -> PeopleBatchDeleteContactCall<'a, S> {
        PeopleBatchDeleteContactCall {
            hub: self.hub,
            _request: request,
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Update a batch of contacts and return a map of resource names to PersonResponses for the updated contacts. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
    /// 
    /// # Arguments
    ///
    /// * `request` - No description provided.
    pub fn batch_update_contacts(&self, request: BatchUpdateContactsRequest) -> PeopleBatchUpdateContactCall<'a, S> {
        PeopleBatchUpdateContactCall {
            hub: self.hub,
            _request: request,
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Create a new contact and return the person resource for that contact. The request returns a 400 error if more than one field is specified on a field that is a singleton for contact sources: * biographies * birthdays * genders * names Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
    /// 
    /// # Arguments
    ///
    /// * `request` - No description provided.
    pub fn create_contact(&self, request: Person) -> PeopleCreateContactCall<'a, S> {
        PeopleCreateContactCall {
            hub: self.hub,
            _request: request,
            _sources: Default::default(),
            _person_fields: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Delete a contact person. Any non-contact data will not be deleted. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
    /// 
    /// # Arguments
    ///
    /// * `resourceName` - Required. The resource name of the contact to delete.
    pub fn delete_contact(&self, resource_name: &str) -> PeopleDeleteContactCall<'a, S> {
        PeopleDeleteContactCall {
            hub: self.hub,
            _resource_name: resource_name.to_string(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Delete a contact's photo. Mutate requests for the same user should be done sequentially to avoid // lock contention.
    /// 
    /// # Arguments
    ///
    /// * `resourceName` - Required. The resource name of the contact whose photo will be deleted.
    pub fn delete_contact_photo(&self, resource_name: &str) -> PeopleDeleteContactPhotoCall<'a, S> {
        PeopleDeleteContactPhotoCall {
            hub: self.hub,
            _resource_name: resource_name.to_string(),
            _sources: Default::default(),
            _person_fields: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Provides information about a person by specifying a resource name. Use `people/me` to indicate the authenticated user. The request returns a 400 error if 'personFields' is not specified.
    /// 
    /// # Arguments
    ///
    /// * `resourceName` - Required. The resource name of the person to provide information about. - To get information about the authenticated user, specify `people/me`. - To get information about a google account, specify `people/{account_id}`. - To get information about a contact, specify the resource name that identifies the contact as returned by `people.connections.list`.
    pub fn get(&self, resource_name: &str) -> PeopleGetCall<'a, S> {
        PeopleGetCall {
            hub: self.hub,
            _resource_name: resource_name.to_string(),
            _sources: Default::default(),
            _request_mask_include_field: Default::default(),
            _person_fields: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Provides information about a list of specific people by specifying a list of requested resource names. Use `people/me` to indicate the authenticated user. The request returns a 400 error if 'personFields' is not specified.
    pub fn get_batch_get(&self) -> PeopleGetBatchGetCall<'a, S> {
        PeopleGetBatchGetCall {
            hub: self.hub,
            _sources: Default::default(),
            _resource_names: Default::default(),
            _request_mask_include_field: Default::default(),
            _person_fields: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Provides a list of domain profiles and domain contacts in the authenticated user's domain directory. When the `sync_token` is specified, resources deleted since the last sync will be returned as a person with `PersonMetadata.deleted` set to true. When the `page_token` or `sync_token` is specified, all other request parameters must match the first call. Writes may have a propagation delay of several minutes for sync requests. Incremental syncs are not intended for read-after-write use cases. See example usage at [List the directory people that have changed](/people/v1/directory#list_the_directory_people_that_have_changed).
    pub fn list_directory_people(&self) -> PeopleListDirectoryPeopleCall<'a, S> {
        PeopleListDirectoryPeopleCall {
            hub: self.hub,
            _sync_token: Default::default(),
            _sources: Default::default(),
            _request_sync_token: Default::default(),
            _read_mask: Default::default(),
            _page_token: Default::default(),
            _page_size: Default::default(),
            _merge_sources: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Provides a list of contacts in the authenticated user's grouped contacts that matches the search query. The query matches on a contact's `names`, `nickNames`, `emailAddresses`, `phoneNumbers`, and `organizations` fields that are from the CONTACT source. **IMPORTANT**: Before searching, clients should send a warmup request with an empty query to update the cache. See https://developers.google.com/people/v1/contacts#search_the_users_contacts
    pub fn search_contacts(&self) -> PeopleSearchContactCall<'a, S> {
        PeopleSearchContactCall {
            hub: self.hub,
            _sources: Default::default(),
            _read_mask: Default::default(),
            _query: Default::default(),
            _page_size: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Provides a list of domain profiles and domain contacts in the authenticated user's domain directory that match the search query.
    pub fn search_directory_people(&self) -> PeopleSearchDirectoryPeopleCall<'a, S> {
        PeopleSearchDirectoryPeopleCall {
            hub: self.hub,
            _sources: Default::default(),
            _read_mask: Default::default(),
            _query: Default::default(),
            _page_token: Default::default(),
            _page_size: Default::default(),
            _merge_sources: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Update contact data for an existing contact person. Any non-contact data will not be modified. Any non-contact data in the person to update will be ignored. All fields specified in the `update_mask` will be replaced. The server returns a 400 error if `person.metadata.sources` is not specified for the contact to be updated or if there is no contact source. The server returns a 400 error with reason `"failedPrecondition"` if `person.metadata.sources.etag` is different than the contact's etag, which indicates the contact has changed since its data was read. Clients should get the latest person and merge their updates into the latest person. The server returns a 400 error if `memberships` are being updated and there are no contact group memberships specified on the person. The server returns a 400 error if more than one field is specified on a field that is a singleton for contact sources: * biographies * birthdays * genders * names Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
    /// 
    /// # Arguments
    ///
    /// * `request` - No description provided.
    /// * `resourceName` - The resource name for the person, assigned by the server. An ASCII string with a max length of 27 characters, in the form of `people/{person_id}`.
    pub fn update_contact(&self, request: Person, resource_name: &str) -> PeopleUpdateContactCall<'a, S> {
        PeopleUpdateContactCall {
            hub: self.hub,
            _request: request,
            _resource_name: resource_name.to_string(),
            _update_person_fields: Default::default(),
            _sources: Default::default(),
            _person_fields: Default::default(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
    
    /// Create a builder to help you perform the following task:
    ///
    /// Update a contact's photo. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
    /// 
    /// # Arguments
    ///
    /// * `request` - No description provided.
    /// * `resourceName` - Required. Person resource name
    pub fn update_contact_photo(&self, request: UpdateContactPhotoRequest, resource_name: &str) -> PeopleUpdateContactPhotoCall<'a, S> {
        PeopleUpdateContactPhotoCall {
            hub: self.hub,
            _request: request,
            _resource_name: resource_name.to_string(),
            _delegate: Default::default(),
            _additional_params: Default::default(),
            _scopes: Default::default(),
        }
    }
}





// ###################
// CallBuilders   ###
// #################

/// Modify the members of a contact group owned by the authenticated user. The only system contact groups that can have members added are `contactGroups/myContacts` and `contactGroups/starred`. Other system contact groups are deprecated and can only have contacts removed.
///
/// A builder for the *members.modify* method supported by a *contactGroup* resource.
/// It is not used directly, but through a `ContactGroupMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// use people1::api::ModifyContactGroupMembersRequest;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // As the method needs a request, you would usually fill it with the desired information
/// // into the respective structure. Some of the parts shown here might not be applicable !
/// // Values shown here are possibly random and not representative !
/// let mut req = ModifyContactGroupMembersRequest::default();
/// 
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.contact_groups().members_modify(req, "resourceName")
///              .doit().await;
/// # }
/// ```
pub struct ContactGroupMemberModifyCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _request: ModifyContactGroupMembersRequest,
    _resource_name: String,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for ContactGroupMemberModifyCall<'a, S> {}

impl<'a, S> ContactGroupMemberModifyCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, ModifyContactGroupMembersResponse)> {
        use url::percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.contactGroups.members.modify",
                               http_method: hyper::Method::POST });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(4 + self._additional_params.len());
        params.push(("resourceName", self._resource_name.to_string()));
        for &field in ["alt", "resourceName"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/{+resourceName}/members:modify";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::Contact.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{+resourceName}", "resourceName")].iter() {
            let mut replace_with = String::new();
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = value.to_string();
                    break;
                }
            }
            if find_this.as_bytes()[1] == '+' as u8 {
                replace_with = percent_encode(replace_with.as_bytes(), DEFAULT_ENCODE_SET).to_string();
            }
            url = url.replace(find_this, &replace_with);
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["resourceName"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();

        let mut json_mime_type: mime::Mime = "application/json".parse().unwrap();
        let mut request_value_reader =
            {
                let mut value = json::value::to_value(&self._request).expect("serde to work");
                client::remove_json_null_values(&mut value);
                let mut dst = io::Cursor::new(Vec::with_capacity(128));
                json::to_writer(&mut dst, &value).unwrap();
                dst
            };
        let request_size = request_value_reader.seek(io::SeekFrom::End(0)).unwrap();
        request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();


        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::POST).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .header(CONTENT_TYPE, format!("{}", json_mime_type.to_string()))
                        .header(CONTENT_LENGTH, request_size as u64)
                        .body(hyper::body::Body::from(request_value_reader.get_ref().clone()));

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    ///
    /// Sets the *request* property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn request(mut self, new_value: ModifyContactGroupMembersRequest) -> ContactGroupMemberModifyCall<'a, S> {
        self._request = new_value;
        self
    }
    /// Required. The resource name of the contact group to modify.
    ///
    /// Sets the *resource name* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn resource_name(mut self, new_value: &str) -> ContactGroupMemberModifyCall<'a, S> {
        self._resource_name = new_value.to_string();
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> ContactGroupMemberModifyCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> ContactGroupMemberModifyCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::Contact`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> ContactGroupMemberModifyCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Get a list of contact groups owned by the authenticated user by specifying a list of contact group resource names.
///
/// A builder for the *batchGet* method supported by a *contactGroup* resource.
/// It is not used directly, but through a `ContactGroupMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.contact_groups().batch_get()
///              .add_resource_names("gubergren")
///              .max_members(-51)
///              .group_fields("gubergren")
///              .doit().await;
/// # }
/// ```
pub struct ContactGroupBatchGetCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _resource_names: Vec<String>,
    _max_members: Option<i32>,
    _group_fields: Option<String>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for ContactGroupBatchGetCall<'a, S> {}

impl<'a, S> ContactGroupBatchGetCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, BatchGetContactGroupsResponse)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.contactGroups.batchGet",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(5 + self._additional_params.len());
        if self._resource_names.len() > 0 {
            for f in self._resource_names.iter() {
                params.push(("resourceNames", f.to_string()));
            }
        }
        if let Some(value) = self._max_members {
            params.push(("maxMembers", value.to_string()));
        }
        if let Some(value) = self._group_fields {
            params.push(("groupFields", value.to_string()));
        }
        for &field in ["alt", "resourceNames", "maxMembers", "groupFields"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/contactGroups:batchGet";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::ContactReadonly.as_ref().to_string(), ());
        }


        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// Required. The resource names of the contact groups to get. There is a maximum of 200 resource names.
    ///
    /// Append the given value to the *resource names* query property.
    /// Each appended value will retain its original ordering and be '/'-separated in the URL's parameters.
    pub fn add_resource_names(mut self, new_value: &str) -> ContactGroupBatchGetCall<'a, S> {
        self._resource_names.push(new_value.to_string());
        self
    }
    /// Optional. Specifies the maximum number of members to return for each group. Defaults to 0 if not set, which will return zero members.
    ///
    /// Sets the *max members* query property to the given value.
    pub fn max_members(mut self, new_value: i32) -> ContactGroupBatchGetCall<'a, S> {
        self._max_members = Some(new_value);
        self
    }
    /// Optional. A field mask to restrict which fields on the group are returned. Defaults to `metadata`, `groupType`, `memberCount`, and `name` if not set or set to empty. Valid fields are: * clientData * groupType * memberCount * metadata * name
    ///
    /// Sets the *group fields* query property to the given value.
    pub fn group_fields(mut self, new_value: &str) -> ContactGroupBatchGetCall<'a, S> {
        self._group_fields = Some(new_value.to_string());
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> ContactGroupBatchGetCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> ContactGroupBatchGetCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::ContactReadonly`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> ContactGroupBatchGetCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Create a new contact group owned by the authenticated user. Created contact group names must be unique to the users contact groups. Attempting to create a group with a duplicate name will return a HTTP 409 error. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
///
/// A builder for the *create* method supported by a *contactGroup* resource.
/// It is not used directly, but through a `ContactGroupMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// use people1::api::CreateContactGroupRequest;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // As the method needs a request, you would usually fill it with the desired information
/// // into the respective structure. Some of the parts shown here might not be applicable !
/// // Values shown here are possibly random and not representative !
/// let mut req = CreateContactGroupRequest::default();
/// 
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.contact_groups().create(req)
///              .doit().await;
/// # }
/// ```
pub struct ContactGroupCreateCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _request: CreateContactGroupRequest,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for ContactGroupCreateCall<'a, S> {}

impl<'a, S> ContactGroupCreateCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, ContactGroup)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.contactGroups.create",
                               http_method: hyper::Method::POST });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(3 + self._additional_params.len());
        for &field in ["alt"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/contactGroups";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::Contact.as_ref().to_string(), ());
        }


        let url = url::Url::parse_with_params(&url, params).unwrap();

        let mut json_mime_type: mime::Mime = "application/json".parse().unwrap();
        let mut request_value_reader =
            {
                let mut value = json::value::to_value(&self._request).expect("serde to work");
                client::remove_json_null_values(&mut value);
                let mut dst = io::Cursor::new(Vec::with_capacity(128));
                json::to_writer(&mut dst, &value).unwrap();
                dst
            };
        let request_size = request_value_reader.seek(io::SeekFrom::End(0)).unwrap();
        request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();


        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::POST).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .header(CONTENT_TYPE, format!("{}", json_mime_type.to_string()))
                        .header(CONTENT_LENGTH, request_size as u64)
                        .body(hyper::body::Body::from(request_value_reader.get_ref().clone()));

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    ///
    /// Sets the *request* property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn request(mut self, new_value: CreateContactGroupRequest) -> ContactGroupCreateCall<'a, S> {
        self._request = new_value;
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> ContactGroupCreateCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> ContactGroupCreateCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::Contact`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> ContactGroupCreateCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Delete an existing contact group owned by the authenticated user by specifying a contact group resource name. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
///
/// A builder for the *delete* method supported by a *contactGroup* resource.
/// It is not used directly, but through a `ContactGroupMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.contact_groups().delete("resourceName")
///              .delete_contacts(true)
///              .doit().await;
/// # }
/// ```
pub struct ContactGroupDeleteCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _resource_name: String,
    _delete_contacts: Option<bool>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for ContactGroupDeleteCall<'a, S> {}

impl<'a, S> ContactGroupDeleteCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, Empty)> {
        use url::percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.contactGroups.delete",
                               http_method: hyper::Method::DELETE });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(4 + self._additional_params.len());
        params.push(("resourceName", self._resource_name.to_string()));
        if let Some(value) = self._delete_contacts {
            params.push(("deleteContacts", value.to_string()));
        }
        for &field in ["alt", "resourceName", "deleteContacts"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/{+resourceName}";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::Contact.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{+resourceName}", "resourceName")].iter() {
            let mut replace_with = String::new();
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = value.to_string();
                    break;
                }
            }
            if find_this.as_bytes()[1] == '+' as u8 {
                replace_with = percent_encode(replace_with.as_bytes(), DEFAULT_ENCODE_SET).to_string();
            }
            url = url.replace(find_this, &replace_with);
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["resourceName"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::DELETE).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// Required. The resource name of the contact group to delete.
    ///
    /// Sets the *resource name* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn resource_name(mut self, new_value: &str) -> ContactGroupDeleteCall<'a, S> {
        self._resource_name = new_value.to_string();
        self
    }
    /// Optional. Set to true to also delete the contacts in the specified group.
    ///
    /// Sets the *delete contacts* query property to the given value.
    pub fn delete_contacts(mut self, new_value: bool) -> ContactGroupDeleteCall<'a, S> {
        self._delete_contacts = Some(new_value);
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> ContactGroupDeleteCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> ContactGroupDeleteCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::Contact`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> ContactGroupDeleteCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Get a specific contact group owned by the authenticated user by specifying a contact group resource name.
///
/// A builder for the *get* method supported by a *contactGroup* resource.
/// It is not used directly, but through a `ContactGroupMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.contact_groups().get("resourceName")
///              .max_members(-47)
///              .group_fields("duo")
///              .doit().await;
/// # }
/// ```
pub struct ContactGroupGetCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _resource_name: String,
    _max_members: Option<i32>,
    _group_fields: Option<String>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for ContactGroupGetCall<'a, S> {}

impl<'a, S> ContactGroupGetCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, ContactGroup)> {
        use url::percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.contactGroups.get",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(5 + self._additional_params.len());
        params.push(("resourceName", self._resource_name.to_string()));
        if let Some(value) = self._max_members {
            params.push(("maxMembers", value.to_string()));
        }
        if let Some(value) = self._group_fields {
            params.push(("groupFields", value.to_string()));
        }
        for &field in ["alt", "resourceName", "maxMembers", "groupFields"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/{+resourceName}";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::ContactReadonly.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{+resourceName}", "resourceName")].iter() {
            let mut replace_with = String::new();
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = value.to_string();
                    break;
                }
            }
            if find_this.as_bytes()[1] == '+' as u8 {
                replace_with = percent_encode(replace_with.as_bytes(), DEFAULT_ENCODE_SET).to_string();
            }
            url = url.replace(find_this, &replace_with);
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["resourceName"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// Required. The resource name of the contact group to get.
    ///
    /// Sets the *resource name* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn resource_name(mut self, new_value: &str) -> ContactGroupGetCall<'a, S> {
        self._resource_name = new_value.to_string();
        self
    }
    /// Optional. Specifies the maximum number of members to return. Defaults to 0 if not set, which will return zero members.
    ///
    /// Sets the *max members* query property to the given value.
    pub fn max_members(mut self, new_value: i32) -> ContactGroupGetCall<'a, S> {
        self._max_members = Some(new_value);
        self
    }
    /// Optional. A field mask to restrict which fields on the group are returned. Defaults to `metadata`, `groupType`, `memberCount`, and `name` if not set or set to empty. Valid fields are: * clientData * groupType * memberCount * metadata * name
    ///
    /// Sets the *group fields* query property to the given value.
    pub fn group_fields(mut self, new_value: &str) -> ContactGroupGetCall<'a, S> {
        self._group_fields = Some(new_value.to_string());
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> ContactGroupGetCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> ContactGroupGetCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::ContactReadonly`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> ContactGroupGetCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// List all contact groups owned by the authenticated user. Members of the contact groups are not populated.
///
/// A builder for the *list* method supported by a *contactGroup* resource.
/// It is not used directly, but through a `ContactGroupMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.contact_groups().list()
///              .sync_token("ipsum")
///              .page_token("sed")
///              .page_size(-37)
///              .group_fields("gubergren")
///              .doit().await;
/// # }
/// ```
pub struct ContactGroupListCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _sync_token: Option<String>,
    _page_token: Option<String>,
    _page_size: Option<i32>,
    _group_fields: Option<String>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for ContactGroupListCall<'a, S> {}

impl<'a, S> ContactGroupListCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, ListContactGroupsResponse)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.contactGroups.list",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(6 + self._additional_params.len());
        if let Some(value) = self._sync_token {
            params.push(("syncToken", value.to_string()));
        }
        if let Some(value) = self._page_token {
            params.push(("pageToken", value.to_string()));
        }
        if let Some(value) = self._page_size {
            params.push(("pageSize", value.to_string()));
        }
        if let Some(value) = self._group_fields {
            params.push(("groupFields", value.to_string()));
        }
        for &field in ["alt", "syncToken", "pageToken", "pageSize", "groupFields"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/contactGroups";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::ContactReadonly.as_ref().to_string(), ());
        }


        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// Optional. A sync token, returned by a previous call to `contactgroups.list`. Only resources changed since the sync token was created will be returned.
    ///
    /// Sets the *sync token* query property to the given value.
    pub fn sync_token(mut self, new_value: &str) -> ContactGroupListCall<'a, S> {
        self._sync_token = Some(new_value.to_string());
        self
    }
    /// Optional. The next_page_token value returned from a previous call to [ListContactGroups](/people/api/rest/v1/contactgroups/list). Requests the next page of resources.
    ///
    /// Sets the *page token* query property to the given value.
    pub fn page_token(mut self, new_value: &str) -> ContactGroupListCall<'a, S> {
        self._page_token = Some(new_value.to_string());
        self
    }
    /// Optional. The maximum number of resources to return. Valid values are between 1 and 1000, inclusive. Defaults to 30 if not set or set to 0.
    ///
    /// Sets the *page size* query property to the given value.
    pub fn page_size(mut self, new_value: i32) -> ContactGroupListCall<'a, S> {
        self._page_size = Some(new_value);
        self
    }
    /// Optional. A field mask to restrict which fields on the group are returned. Defaults to `metadata`, `groupType`, `memberCount`, and `name` if not set or set to empty. Valid fields are: * clientData * groupType * memberCount * metadata * name
    ///
    /// Sets the *group fields* query property to the given value.
    pub fn group_fields(mut self, new_value: &str) -> ContactGroupListCall<'a, S> {
        self._group_fields = Some(new_value.to_string());
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> ContactGroupListCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> ContactGroupListCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::ContactReadonly`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> ContactGroupListCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Update the name of an existing contact group owned by the authenticated user. Updated contact group names must be unique to the users contact groups. Attempting to create a group with a duplicate name will return a HTTP 409 error. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
///
/// A builder for the *update* method supported by a *contactGroup* resource.
/// It is not used directly, but through a `ContactGroupMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// use people1::api::UpdateContactGroupRequest;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // As the method needs a request, you would usually fill it with the desired information
/// // into the respective structure. Some of the parts shown here might not be applicable !
/// // Values shown here are possibly random and not representative !
/// let mut req = UpdateContactGroupRequest::default();
/// 
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.contact_groups().update(req, "resourceName")
///              .doit().await;
/// # }
/// ```
pub struct ContactGroupUpdateCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _request: UpdateContactGroupRequest,
    _resource_name: String,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for ContactGroupUpdateCall<'a, S> {}

impl<'a, S> ContactGroupUpdateCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, ContactGroup)> {
        use url::percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.contactGroups.update",
                               http_method: hyper::Method::PUT });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(4 + self._additional_params.len());
        params.push(("resourceName", self._resource_name.to_string()));
        for &field in ["alt", "resourceName"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/{+resourceName}";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::Contact.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{+resourceName}", "resourceName")].iter() {
            let mut replace_with = String::new();
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = value.to_string();
                    break;
                }
            }
            if find_this.as_bytes()[1] == '+' as u8 {
                replace_with = percent_encode(replace_with.as_bytes(), DEFAULT_ENCODE_SET).to_string();
            }
            url = url.replace(find_this, &replace_with);
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["resourceName"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();

        let mut json_mime_type: mime::Mime = "application/json".parse().unwrap();
        let mut request_value_reader =
            {
                let mut value = json::value::to_value(&self._request).expect("serde to work");
                client::remove_json_null_values(&mut value);
                let mut dst = io::Cursor::new(Vec::with_capacity(128));
                json::to_writer(&mut dst, &value).unwrap();
                dst
            };
        let request_size = request_value_reader.seek(io::SeekFrom::End(0)).unwrap();
        request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();


        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::PUT).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .header(CONTENT_TYPE, format!("{}", json_mime_type.to_string()))
                        .header(CONTENT_LENGTH, request_size as u64)
                        .body(hyper::body::Body::from(request_value_reader.get_ref().clone()));

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    ///
    /// Sets the *request* property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn request(mut self, new_value: UpdateContactGroupRequest) -> ContactGroupUpdateCall<'a, S> {
        self._request = new_value;
        self
    }
    /// The resource name for the contact group, assigned by the server. An ASCII string, in the form of `contactGroups/{contact_group_id}`.
    ///
    /// Sets the *resource name* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn resource_name(mut self, new_value: &str) -> ContactGroupUpdateCall<'a, S> {
        self._resource_name = new_value.to_string();
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> ContactGroupUpdateCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> ContactGroupUpdateCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::Contact`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> ContactGroupUpdateCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Copies an "Other contact" to a new contact in the user's "myContacts" group Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
///
/// A builder for the *copyOtherContactToMyContactsGroup* method supported by a *otherContact* resource.
/// It is not used directly, but through a `OtherContactMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// use people1::api::CopyOtherContactToMyContactsGroupRequest;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // As the method needs a request, you would usually fill it with the desired information
/// // into the respective structure. Some of the parts shown here might not be applicable !
/// // Values shown here are possibly random and not representative !
/// let mut req = CopyOtherContactToMyContactsGroupRequest::default();
/// 
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.other_contacts().copy_other_contact_to_my_contacts_group(req, "resourceName")
///              .doit().await;
/// # }
/// ```
pub struct OtherContactCopyOtherContactToMyContactsGroupCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _request: CopyOtherContactToMyContactsGroupRequest,
    _resource_name: String,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for OtherContactCopyOtherContactToMyContactsGroupCall<'a, S> {}

impl<'a, S> OtherContactCopyOtherContactToMyContactsGroupCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, Person)> {
        use url::percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.otherContacts.copyOtherContactToMyContactsGroup",
                               http_method: hyper::Method::POST });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(4 + self._additional_params.len());
        params.push(("resourceName", self._resource_name.to_string()));
        for &field in ["alt", "resourceName"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/{+resourceName}:copyOtherContactToMyContactsGroup";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::Contact.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{+resourceName}", "resourceName")].iter() {
            let mut replace_with = String::new();
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = value.to_string();
                    break;
                }
            }
            if find_this.as_bytes()[1] == '+' as u8 {
                replace_with = percent_encode(replace_with.as_bytes(), DEFAULT_ENCODE_SET).to_string();
            }
            url = url.replace(find_this, &replace_with);
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["resourceName"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();

        let mut json_mime_type: mime::Mime = "application/json".parse().unwrap();
        let mut request_value_reader =
            {
                let mut value = json::value::to_value(&self._request).expect("serde to work");
                client::remove_json_null_values(&mut value);
                let mut dst = io::Cursor::new(Vec::with_capacity(128));
                json::to_writer(&mut dst, &value).unwrap();
                dst
            };
        let request_size = request_value_reader.seek(io::SeekFrom::End(0)).unwrap();
        request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();


        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::POST).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .header(CONTENT_TYPE, format!("{}", json_mime_type.to_string()))
                        .header(CONTENT_LENGTH, request_size as u64)
                        .body(hyper::body::Body::from(request_value_reader.get_ref().clone()));

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    ///
    /// Sets the *request* property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn request(mut self, new_value: CopyOtherContactToMyContactsGroupRequest) -> OtherContactCopyOtherContactToMyContactsGroupCall<'a, S> {
        self._request = new_value;
        self
    }
    /// Required. The resource name of the "Other contact" to copy.
    ///
    /// Sets the *resource name* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn resource_name(mut self, new_value: &str) -> OtherContactCopyOtherContactToMyContactsGroupCall<'a, S> {
        self._resource_name = new_value.to_string();
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> OtherContactCopyOtherContactToMyContactsGroupCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> OtherContactCopyOtherContactToMyContactsGroupCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::Contact`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> OtherContactCopyOtherContactToMyContactsGroupCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// List all "Other contacts", that is contacts that are not in a contact group. "Other contacts" are typically auto created contacts from interactions. Sync tokens expire 7 days after the full sync. A request with an expired sync token will result in a 410 error. In the case of such an error clients should make a full sync request without a `sync_token`. The first page of a full sync request has an additional quota. If the quota is exceeded, a 429 error will be returned. This quota is fixed and can not be increased. When the `sync_token` is specified, resources deleted since the last sync will be returned as a person with `PersonMetadata.deleted` set to true. When the `page_token` or `sync_token` is specified, all other request parameters must match the first call. Writes may have a propagation delay of several minutes for sync requests. Incremental syncs are not intended for read-after-write use cases. See example usage at [List the user's other contacts that have changed](/people/v1/other-contacts#list_the_users_other_contacts_that_have_changed).
///
/// A builder for the *list* method supported by a *otherContact* resource.
/// It is not used directly, but through a `OtherContactMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.other_contacts().list()
///              .sync_token("ipsum")
///              .add_sources("ipsum")
///              .request_sync_token(true)
///              .read_mask("ea")
///              .page_token("dolor")
///              .page_size(-56)
///              .doit().await;
/// # }
/// ```
pub struct OtherContactListCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _sync_token: Option<String>,
    _sources: Vec<String>,
    _request_sync_token: Option<bool>,
    _read_mask: Option<String>,
    _page_token: Option<String>,
    _page_size: Option<i32>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for OtherContactListCall<'a, S> {}

impl<'a, S> OtherContactListCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, ListOtherContactsResponse)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.otherContacts.list",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(8 + self._additional_params.len());
        if let Some(value) = self._sync_token {
            params.push(("syncToken", value.to_string()));
        }
        if self._sources.len() > 0 {
            for f in self._sources.iter() {
                params.push(("sources", f.to_string()));
            }
        }
        if let Some(value) = self._request_sync_token {
            params.push(("requestSyncToken", value.to_string()));
        }
        if let Some(value) = self._read_mask {
            params.push(("readMask", value.to_string()));
        }
        if let Some(value) = self._page_token {
            params.push(("pageToken", value.to_string()));
        }
        if let Some(value) = self._page_size {
            params.push(("pageSize", value.to_string()));
        }
        for &field in ["alt", "syncToken", "sources", "requestSyncToken", "readMask", "pageToken", "pageSize"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/otherContacts";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::ContactOtherReadonly.as_ref().to_string(), ());
        }


        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// Optional. A sync token, received from a previous response `next_sync_token` Provide this to retrieve only the resources changed since the last request. When syncing, all other parameters provided to `otherContacts.list` must match the first call that provided the sync token. More details about sync behavior at `otherContacts.list`.
    ///
    /// Sets the *sync token* query property to the given value.
    pub fn sync_token(mut self, new_value: &str) -> OtherContactListCall<'a, S> {
        self._sync_token = Some(new_value.to_string());
        self
    }
    /// Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT if not set.
    ///
    /// Append the given value to the *sources* query property.
    /// Each appended value will retain its original ordering and be '/'-separated in the URL's parameters.
    pub fn add_sources(mut self, new_value: &str) -> OtherContactListCall<'a, S> {
        self._sources.push(new_value.to_string());
        self
    }
    /// Optional. Whether the response should return `next_sync_token` on the last page of results. It can be used to get incremental changes since the last request by setting it on the request `sync_token`. More details about sync behavior at `otherContacts.list`.
    ///
    /// Sets the *request sync token* query property to the given value.
    pub fn request_sync_token(mut self, new_value: bool) -> OtherContactListCall<'a, S> {
        self._request_sync_token = Some(new_value);
        self
    }
    /// Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. What values are valid depend on what ReadSourceType is used. If READ_SOURCE_TYPE_CONTACT is used, valid values are: * emailAddresses * metadata * names * phoneNumbers * photos If READ_SOURCE_TYPE_PROFILE is used, valid values are: * addresses * ageRanges * biographies * birthdays * calendarUrls * clientData * coverPhotos * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * metadata * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * photos * relations * sipAddresses * skills * urls * userDefined
    ///
    /// Sets the *read mask* query property to the given value.
    pub fn read_mask(mut self, new_value: &str) -> OtherContactListCall<'a, S> {
        self._read_mask = Some(new_value.to_string());
        self
    }
    /// Optional. A page token, received from a previous response `next_page_token`. Provide this to retrieve the subsequent page. When paginating, all other parameters provided to `otherContacts.list` must match the first call that provided the page token.
    ///
    /// Sets the *page token* query property to the given value.
    pub fn page_token(mut self, new_value: &str) -> OtherContactListCall<'a, S> {
        self._page_token = Some(new_value.to_string());
        self
    }
    /// Optional. The number of "Other contacts" to include in the response. Valid values are between 1 and 1000, inclusive. Defaults to 100 if not set or set to 0.
    ///
    /// Sets the *page size* query property to the given value.
    pub fn page_size(mut self, new_value: i32) -> OtherContactListCall<'a, S> {
        self._page_size = Some(new_value);
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> OtherContactListCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> OtherContactListCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::ContactOtherReadonly`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> OtherContactListCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Provides a list of contacts in the authenticated user's other contacts that matches the search query. The query matches on a contact's `names`, `emailAddresses`, and `phoneNumbers` fields that are from the OTHER_CONTACT source. **IMPORTANT**: Before searching, clients should send a warmup request with an empty query to update the cache. See https://developers.google.com/people/v1/other-contacts#search_the_users_other_contacts
///
/// A builder for the *search* method supported by a *otherContact* resource.
/// It is not used directly, but through a `OtherContactMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.other_contacts().search()
///              .read_mask("eos")
///              .query("labore")
///              .page_size(-43)
///              .doit().await;
/// # }
/// ```
pub struct OtherContactSearchCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _read_mask: Option<String>,
    _query: Option<String>,
    _page_size: Option<i32>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for OtherContactSearchCall<'a, S> {}

impl<'a, S> OtherContactSearchCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, SearchResponse)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.otherContacts.search",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(5 + self._additional_params.len());
        if let Some(value) = self._read_mask {
            params.push(("readMask", value.to_string()));
        }
        if let Some(value) = self._query {
            params.push(("query", value.to_string()));
        }
        if let Some(value) = self._page_size {
            params.push(("pageSize", value.to_string()));
        }
        for &field in ["alt", "readMask", "query", "pageSize"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/otherContacts:search";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::ContactOtherReadonly.as_ref().to_string(), ());
        }


        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Valid values are: * emailAddresses * metadata * names * phoneNumbers
    ///
    /// Sets the *read mask* query property to the given value.
    pub fn read_mask(mut self, new_value: &str) -> OtherContactSearchCall<'a, S> {
        self._read_mask = Some(new_value.to_string());
        self
    }
    /// Required. The plain-text query for the request. The query is used to match prefix phrases of the fields on a person. For example, a person with name "foo name" matches queries such as "f", "fo", "foo", "foo n", "nam", etc., but not "oo n".
    ///
    /// Sets the *query* query property to the given value.
    pub fn query(mut self, new_value: &str) -> OtherContactSearchCall<'a, S> {
        self._query = Some(new_value.to_string());
        self
    }
    /// Optional. The number of results to return. Defaults to 10 if field is not set, or set to 0. Values greater than 30 will be capped to 30.
    ///
    /// Sets the *page size* query property to the given value.
    pub fn page_size(mut self, new_value: i32) -> OtherContactSearchCall<'a, S> {
        self._page_size = Some(new_value);
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> OtherContactSearchCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> OtherContactSearchCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::ContactOtherReadonly`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> OtherContactSearchCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Provides a list of the authenticated user's contacts. Sync tokens expire 7 days after the full sync. A request with an expired sync token will result in a 410 error. In the case of such an error clients should make a full sync request without a `sync_token`. The first page of a full sync request has an additional quota. If the quota is exceeded, a 429 error will be returned. This quota is fixed and can not be increased. When the `sync_token` is specified, resources deleted since the last sync will be returned as a person with `PersonMetadata.deleted` set to true. When the `page_token` or `sync_token` is specified, all other request parameters must match the first call. Writes may have a propagation delay of several minutes for sync requests. Incremental syncs are not intended for read-after-write use cases. See example usage at [List the user's contacts that have changed](/people/v1/contacts#list_the_users_contacts_that_have_changed).
///
/// A builder for the *connections.list* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().connections_list("resourceName")
///              .sync_token("sed")
///              .add_sources("no")
///              .sort_order("Stet")
///              .request_sync_token(true)
///              .request_mask_include_field("et")
///              .person_fields("et")
///              .page_token("vero")
///              .page_size(-31)
///              .doit().await;
/// # }
/// ```
pub struct PeopleConnectionListCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _resource_name: String,
    _sync_token: Option<String>,
    _sources: Vec<String>,
    _sort_order: Option<String>,
    _request_sync_token: Option<bool>,
    _request_mask_include_field: Option<String>,
    _person_fields: Option<String>,
    _page_token: Option<String>,
    _page_size: Option<i32>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for PeopleConnectionListCall<'a, S> {}

impl<'a, S> PeopleConnectionListCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, ListConnectionsResponse)> {
        use url::percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.people.connections.list",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(11 + self._additional_params.len());
        params.push(("resourceName", self._resource_name.to_string()));
        if let Some(value) = self._sync_token {
            params.push(("syncToken", value.to_string()));
        }
        if self._sources.len() > 0 {
            for f in self._sources.iter() {
                params.push(("sources", f.to_string()));
            }
        }
        if let Some(value) = self._sort_order {
            params.push(("sortOrder", value.to_string()));
        }
        if let Some(value) = self._request_sync_token {
            params.push(("requestSyncToken", value.to_string()));
        }
        if let Some(value) = self._request_mask_include_field {
            params.push(("requestMask.includeField", value.to_string()));
        }
        if let Some(value) = self._person_fields {
            params.push(("personFields", value.to_string()));
        }
        if let Some(value) = self._page_token {
            params.push(("pageToken", value.to_string()));
        }
        if let Some(value) = self._page_size {
            params.push(("pageSize", value.to_string()));
        }
        for &field in ["alt", "resourceName", "syncToken", "sources", "sortOrder", "requestSyncToken", "requestMask.includeField", "personFields", "pageToken", "pageSize"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/{+resourceName}/connections";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::ContactReadonly.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{+resourceName}", "resourceName")].iter() {
            let mut replace_with = String::new();
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = value.to_string();
                    break;
                }
            }
            if find_this.as_bytes()[1] == '+' as u8 {
                replace_with = percent_encode(replace_with.as_bytes(), DEFAULT_ENCODE_SET).to_string();
            }
            url = url.replace(find_this, &replace_with);
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["resourceName"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// Required. The resource name to return connections for. Only `people/me` is valid.
    ///
    /// Sets the *resource name* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn resource_name(mut self, new_value: &str) -> PeopleConnectionListCall<'a, S> {
        self._resource_name = new_value.to_string();
        self
    }
    /// Optional. A sync token, received from a previous response `next_sync_token` Provide this to retrieve only the resources changed since the last request. When syncing, all other parameters provided to `people.connections.list` must match the first call that provided the sync token. More details about sync behavior at `people.connections.list`.
    ///
    /// Sets the *sync token* query property to the given value.
    pub fn sync_token(mut self, new_value: &str) -> PeopleConnectionListCall<'a, S> {
        self._sync_token = Some(new_value.to_string());
        self
    }
    /// Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT and READ_SOURCE_TYPE_PROFILE if not set.
    ///
    /// Append the given value to the *sources* query property.
    /// Each appended value will retain its original ordering and be '/'-separated in the URL's parameters.
    pub fn add_sources(mut self, new_value: &str) -> PeopleConnectionListCall<'a, S> {
        self._sources.push(new_value.to_string());
        self
    }
    /// Optional. The order in which the connections should be sorted. Defaults to `LAST_MODIFIED_ASCENDING`.
    ///
    /// Sets the *sort order* query property to the given value.
    pub fn sort_order(mut self, new_value: &str) -> PeopleConnectionListCall<'a, S> {
        self._sort_order = Some(new_value.to_string());
        self
    }
    /// Optional. Whether the response should return `next_sync_token` on the last page of results. It can be used to get incremental changes since the last request by setting it on the request `sync_token`. More details about sync behavior at `people.connections.list`.
    ///
    /// Sets the *request sync token* query property to the given value.
    pub fn request_sync_token(mut self, new_value: bool) -> PeopleConnectionListCall<'a, S> {
        self._request_sync_token = Some(new_value);
        self
    }
    /// Required. Comma-separated list of person fields to be included in the response. Each path should start with `person.`: for example, `person.names` or `person.photos`.
    ///
    /// Sets the *request mask.include field* query property to the given value.
    pub fn request_mask_include_field(mut self, new_value: &str) -> PeopleConnectionListCall<'a, S> {
        self._request_mask_include_field = Some(new_value.to_string());
        self
    }
    /// Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Valid values are: * addresses * ageRanges * biographies * birthdays * calendarUrls * clientData * coverPhotos * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * metadata * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * photos * relations * sipAddresses * skills * urls * userDefined
    ///
    /// Sets the *person fields* query property to the given value.
    pub fn person_fields(mut self, new_value: &str) -> PeopleConnectionListCall<'a, S> {
        self._person_fields = Some(new_value.to_string());
        self
    }
    /// Optional. A page token, received from a previous response `next_page_token`. Provide this to retrieve the subsequent page. When paginating, all other parameters provided to `people.connections.list` must match the first call that provided the page token.
    ///
    /// Sets the *page token* query property to the given value.
    pub fn page_token(mut self, new_value: &str) -> PeopleConnectionListCall<'a, S> {
        self._page_token = Some(new_value.to_string());
        self
    }
    /// Optional. The number of connections to include in the response. Valid values are between 1 and 1000, inclusive. Defaults to 100 if not set or set to 0.
    ///
    /// Sets the *page size* query property to the given value.
    pub fn page_size(mut self, new_value: i32) -> PeopleConnectionListCall<'a, S> {
        self._page_size = Some(new_value);
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleConnectionListCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PeopleConnectionListCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::ContactReadonly`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> PeopleConnectionListCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Create a batch of new contacts and return the PersonResponses for the newly Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
///
/// A builder for the *batchCreateContacts* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// use people1::api::BatchCreateContactsRequest;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // As the method needs a request, you would usually fill it with the desired information
/// // into the respective structure. Some of the parts shown here might not be applicable !
/// // Values shown here are possibly random and not representative !
/// let mut req = BatchCreateContactsRequest::default();
/// 
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().batch_create_contacts(req)
///              .doit().await;
/// # }
/// ```
pub struct PeopleBatchCreateContactCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _request: BatchCreateContactsRequest,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for PeopleBatchCreateContactCall<'a, S> {}

impl<'a, S> PeopleBatchCreateContactCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, BatchCreateContactsResponse)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.people.batchCreateContacts",
                               http_method: hyper::Method::POST });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(3 + self._additional_params.len());
        for &field in ["alt"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/people:batchCreateContacts";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::Contact.as_ref().to_string(), ());
        }


        let url = url::Url::parse_with_params(&url, params).unwrap();

        let mut json_mime_type: mime::Mime = "application/json".parse().unwrap();
        let mut request_value_reader =
            {
                let mut value = json::value::to_value(&self._request).expect("serde to work");
                client::remove_json_null_values(&mut value);
                let mut dst = io::Cursor::new(Vec::with_capacity(128));
                json::to_writer(&mut dst, &value).unwrap();
                dst
            };
        let request_size = request_value_reader.seek(io::SeekFrom::End(0)).unwrap();
        request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();


        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::POST).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .header(CONTENT_TYPE, format!("{}", json_mime_type.to_string()))
                        .header(CONTENT_LENGTH, request_size as u64)
                        .body(hyper::body::Body::from(request_value_reader.get_ref().clone()));

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    ///
    /// Sets the *request* property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn request(mut self, new_value: BatchCreateContactsRequest) -> PeopleBatchCreateContactCall<'a, S> {
        self._request = new_value;
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleBatchCreateContactCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PeopleBatchCreateContactCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::Contact`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> PeopleBatchCreateContactCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Delete a batch of contacts. Any non-contact data will not be deleted. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
///
/// A builder for the *batchDeleteContacts* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// use people1::api::BatchDeleteContactsRequest;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // As the method needs a request, you would usually fill it with the desired information
/// // into the respective structure. Some of the parts shown here might not be applicable !
/// // Values shown here are possibly random and not representative !
/// let mut req = BatchDeleteContactsRequest::default();
/// 
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().batch_delete_contacts(req)
///              .doit().await;
/// # }
/// ```
pub struct PeopleBatchDeleteContactCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _request: BatchDeleteContactsRequest,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for PeopleBatchDeleteContactCall<'a, S> {}

impl<'a, S> PeopleBatchDeleteContactCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, Empty)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.people.batchDeleteContacts",
                               http_method: hyper::Method::POST });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(3 + self._additional_params.len());
        for &field in ["alt"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/people:batchDeleteContacts";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::Contact.as_ref().to_string(), ());
        }


        let url = url::Url::parse_with_params(&url, params).unwrap();

        let mut json_mime_type: mime::Mime = "application/json".parse().unwrap();
        let mut request_value_reader =
            {
                let mut value = json::value::to_value(&self._request).expect("serde to work");
                client::remove_json_null_values(&mut value);
                let mut dst = io::Cursor::new(Vec::with_capacity(128));
                json::to_writer(&mut dst, &value).unwrap();
                dst
            };
        let request_size = request_value_reader.seek(io::SeekFrom::End(0)).unwrap();
        request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();


        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::POST).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .header(CONTENT_TYPE, format!("{}", json_mime_type.to_string()))
                        .header(CONTENT_LENGTH, request_size as u64)
                        .body(hyper::body::Body::from(request_value_reader.get_ref().clone()));

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    ///
    /// Sets the *request* property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn request(mut self, new_value: BatchDeleteContactsRequest) -> PeopleBatchDeleteContactCall<'a, S> {
        self._request = new_value;
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleBatchDeleteContactCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PeopleBatchDeleteContactCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::Contact`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> PeopleBatchDeleteContactCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Update a batch of contacts and return a map of resource names to PersonResponses for the updated contacts. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
///
/// A builder for the *batchUpdateContacts* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// use people1::api::BatchUpdateContactsRequest;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // As the method needs a request, you would usually fill it with the desired information
/// // into the respective structure. Some of the parts shown here might not be applicable !
/// // Values shown here are possibly random and not representative !
/// let mut req = BatchUpdateContactsRequest::default();
/// 
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().batch_update_contacts(req)
///              .doit().await;
/// # }
/// ```
pub struct PeopleBatchUpdateContactCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _request: BatchUpdateContactsRequest,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for PeopleBatchUpdateContactCall<'a, S> {}

impl<'a, S> PeopleBatchUpdateContactCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, BatchUpdateContactsResponse)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.people.batchUpdateContacts",
                               http_method: hyper::Method::POST });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(3 + self._additional_params.len());
        for &field in ["alt"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/people:batchUpdateContacts";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::Contact.as_ref().to_string(), ());
        }


        let url = url::Url::parse_with_params(&url, params).unwrap();

        let mut json_mime_type: mime::Mime = "application/json".parse().unwrap();
        let mut request_value_reader =
            {
                let mut value = json::value::to_value(&self._request).expect("serde to work");
                client::remove_json_null_values(&mut value);
                let mut dst = io::Cursor::new(Vec::with_capacity(128));
                json::to_writer(&mut dst, &value).unwrap();
                dst
            };
        let request_size = request_value_reader.seek(io::SeekFrom::End(0)).unwrap();
        request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();


        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::POST).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .header(CONTENT_TYPE, format!("{}", json_mime_type.to_string()))
                        .header(CONTENT_LENGTH, request_size as u64)
                        .body(hyper::body::Body::from(request_value_reader.get_ref().clone()));

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    ///
    /// Sets the *request* property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn request(mut self, new_value: BatchUpdateContactsRequest) -> PeopleBatchUpdateContactCall<'a, S> {
        self._request = new_value;
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleBatchUpdateContactCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PeopleBatchUpdateContactCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::Contact`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> PeopleBatchUpdateContactCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Create a new contact and return the person resource for that contact. The request returns a 400 error if more than one field is specified on a field that is a singleton for contact sources: * biographies * birthdays * genders * names Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
///
/// A builder for the *createContact* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// use people1::api::Person;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // As the method needs a request, you would usually fill it with the desired information
/// // into the respective structure. Some of the parts shown here might not be applicable !
/// // Values shown here are possibly random and not representative !
/// let mut req = Person::default();
/// 
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().create_contact(req)
///              .add_sources("sed")
///              .person_fields("duo")
///              .doit().await;
/// # }
/// ```
pub struct PeopleCreateContactCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _request: Person,
    _sources: Vec<String>,
    _person_fields: Option<String>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for PeopleCreateContactCall<'a, S> {}

impl<'a, S> PeopleCreateContactCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, Person)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.people.createContact",
                               http_method: hyper::Method::POST });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(5 + self._additional_params.len());
        if self._sources.len() > 0 {
            for f in self._sources.iter() {
                params.push(("sources", f.to_string()));
            }
        }
        if let Some(value) = self._person_fields {
            params.push(("personFields", value.to_string()));
        }
        for &field in ["alt", "sources", "personFields"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/people:createContact";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::Contact.as_ref().to_string(), ());
        }


        let url = url::Url::parse_with_params(&url, params).unwrap();

        let mut json_mime_type: mime::Mime = "application/json".parse().unwrap();
        let mut request_value_reader =
            {
                let mut value = json::value::to_value(&self._request).expect("serde to work");
                client::remove_json_null_values(&mut value);
                let mut dst = io::Cursor::new(Vec::with_capacity(128));
                json::to_writer(&mut dst, &value).unwrap();
                dst
            };
        let request_size = request_value_reader.seek(io::SeekFrom::End(0)).unwrap();
        request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();


        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::POST).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .header(CONTENT_TYPE, format!("{}", json_mime_type.to_string()))
                        .header(CONTENT_LENGTH, request_size as u64)
                        .body(hyper::body::Body::from(request_value_reader.get_ref().clone()));

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    ///
    /// Sets the *request* property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn request(mut self, new_value: Person) -> PeopleCreateContactCall<'a, S> {
        self._request = new_value;
        self
    }
    /// Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT and READ_SOURCE_TYPE_PROFILE if not set.
    ///
    /// Append the given value to the *sources* query property.
    /// Each appended value will retain its original ordering and be '/'-separated in the URL's parameters.
    pub fn add_sources(mut self, new_value: &str) -> PeopleCreateContactCall<'a, S> {
        self._sources.push(new_value.to_string());
        self
    }
    /// Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Defaults to all fields if not set. Valid values are: * addresses * ageRanges * biographies * birthdays * calendarUrls * clientData * coverPhotos * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * metadata * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * photos * relations * sipAddresses * skills * urls * userDefined
    ///
    /// Sets the *person fields* query property to the given value.
    pub fn person_fields(mut self, new_value: &str) -> PeopleCreateContactCall<'a, S> {
        self._person_fields = Some(new_value.to_string());
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleCreateContactCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PeopleCreateContactCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::Contact`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> PeopleCreateContactCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Delete a contact person. Any non-contact data will not be deleted. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
///
/// A builder for the *deleteContact* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().delete_contact("resourceName")
///              .doit().await;
/// # }
/// ```
pub struct PeopleDeleteContactCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _resource_name: String,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for PeopleDeleteContactCall<'a, S> {}

impl<'a, S> PeopleDeleteContactCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, Empty)> {
        use url::percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.people.deleteContact",
                               http_method: hyper::Method::DELETE });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(3 + self._additional_params.len());
        params.push(("resourceName", self._resource_name.to_string()));
        for &field in ["alt", "resourceName"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/{+resourceName}:deleteContact";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::Contact.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{+resourceName}", "resourceName")].iter() {
            let mut replace_with = String::new();
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = value.to_string();
                    break;
                }
            }
            if find_this.as_bytes()[1] == '+' as u8 {
                replace_with = percent_encode(replace_with.as_bytes(), DEFAULT_ENCODE_SET).to_string();
            }
            url = url.replace(find_this, &replace_with);
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["resourceName"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::DELETE).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// Required. The resource name of the contact to delete.
    ///
    /// Sets the *resource name* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn resource_name(mut self, new_value: &str) -> PeopleDeleteContactCall<'a, S> {
        self._resource_name = new_value.to_string();
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleDeleteContactCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PeopleDeleteContactCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::Contact`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> PeopleDeleteContactCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Delete a contact's photo. Mutate requests for the same user should be done sequentially to avoid // lock contention.
///
/// A builder for the *deleteContactPhoto* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().delete_contact_photo("resourceName")
///              .add_sources("voluptua.")
///              .person_fields("amet.")
///              .doit().await;
/// # }
/// ```
pub struct PeopleDeleteContactPhotoCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _resource_name: String,
    _sources: Vec<String>,
    _person_fields: Option<String>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for PeopleDeleteContactPhotoCall<'a, S> {}

impl<'a, S> PeopleDeleteContactPhotoCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, DeleteContactPhotoResponse)> {
        use url::percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.people.deleteContactPhoto",
                               http_method: hyper::Method::DELETE });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(5 + self._additional_params.len());
        params.push(("resourceName", self._resource_name.to_string()));
        if self._sources.len() > 0 {
            for f in self._sources.iter() {
                params.push(("sources", f.to_string()));
            }
        }
        if let Some(value) = self._person_fields {
            params.push(("personFields", value.to_string()));
        }
        for &field in ["alt", "resourceName", "sources", "personFields"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/{+resourceName}:deleteContactPhoto";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::Contact.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{+resourceName}", "resourceName")].iter() {
            let mut replace_with = String::new();
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = value.to_string();
                    break;
                }
            }
            if find_this.as_bytes()[1] == '+' as u8 {
                replace_with = percent_encode(replace_with.as_bytes(), DEFAULT_ENCODE_SET).to_string();
            }
            url = url.replace(find_this, &replace_with);
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["resourceName"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::DELETE).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// Required. The resource name of the contact whose photo will be deleted.
    ///
    /// Sets the *resource name* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn resource_name(mut self, new_value: &str) -> PeopleDeleteContactPhotoCall<'a, S> {
        self._resource_name = new_value.to_string();
        self
    }
    /// Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT and READ_SOURCE_TYPE_PROFILE if not set.
    ///
    /// Append the given value to the *sources* query property.
    /// Each appended value will retain its original ordering and be '/'-separated in the URL's parameters.
    pub fn add_sources(mut self, new_value: &str) -> PeopleDeleteContactPhotoCall<'a, S> {
        self._sources.push(new_value.to_string());
        self
    }
    /// Optional. A field mask to restrict which fields on the person are returned. Multiple fields can be specified by separating them with commas. Defaults to empty if not set, which will skip the post mutate get. Valid values are: * addresses * ageRanges * biographies * birthdays * calendarUrls * clientData * coverPhotos * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * metadata * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * photos * relations * sipAddresses * skills * urls * userDefined
    ///
    /// Sets the *person fields* query property to the given value.
    pub fn person_fields(mut self, new_value: &str) -> PeopleDeleteContactPhotoCall<'a, S> {
        self._person_fields = Some(new_value.to_string());
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleDeleteContactPhotoCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PeopleDeleteContactPhotoCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::Contact`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> PeopleDeleteContactPhotoCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Provides information about a person by specifying a resource name. Use `people/me` to indicate the authenticated user. The request returns a 400 error if 'personFields' is not specified.
///
/// A builder for the *get* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().get("resourceName")
///              .add_sources("diam")
///              .request_mask_include_field("dolor")
///              .person_fields("et")
///              .doit().await;
/// # }
/// ```
pub struct PeopleGetCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _resource_name: String,
    _sources: Vec<String>,
    _request_mask_include_field: Option<String>,
    _person_fields: Option<String>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for PeopleGetCall<'a, S> {}

impl<'a, S> PeopleGetCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, Person)> {
        use url::percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.people.get",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(6 + self._additional_params.len());
        params.push(("resourceName", self._resource_name.to_string()));
        if self._sources.len() > 0 {
            for f in self._sources.iter() {
                params.push(("sources", f.to_string()));
            }
        }
        if let Some(value) = self._request_mask_include_field {
            params.push(("requestMask.includeField", value.to_string()));
        }
        if let Some(value) = self._person_fields {
            params.push(("personFields", value.to_string()));
        }
        for &field in ["alt", "resourceName", "sources", "requestMask.includeField", "personFields"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/{+resourceName}";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::ContactReadonly.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{+resourceName}", "resourceName")].iter() {
            let mut replace_with = String::new();
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = value.to_string();
                    break;
                }
            }
            if find_this.as_bytes()[1] == '+' as u8 {
                replace_with = percent_encode(replace_with.as_bytes(), DEFAULT_ENCODE_SET).to_string();
            }
            url = url.replace(find_this, &replace_with);
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["resourceName"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// Required. The resource name of the person to provide information about. - To get information about the authenticated user, specify `people/me`. - To get information about a google account, specify `people/{account_id}`. - To get information about a contact, specify the resource name that identifies the contact as returned by `people.connections.list`.
    ///
    /// Sets the *resource name* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn resource_name(mut self, new_value: &str) -> PeopleGetCall<'a, S> {
        self._resource_name = new_value.to_string();
        self
    }
    /// Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_PROFILE and READ_SOURCE_TYPE_CONTACT if not set.
    ///
    /// Append the given value to the *sources* query property.
    /// Each appended value will retain its original ordering and be '/'-separated in the URL's parameters.
    pub fn add_sources(mut self, new_value: &str) -> PeopleGetCall<'a, S> {
        self._sources.push(new_value.to_string());
        self
    }
    /// Required. Comma-separated list of person fields to be included in the response. Each path should start with `person.`: for example, `person.names` or `person.photos`.
    ///
    /// Sets the *request mask.include field* query property to the given value.
    pub fn request_mask_include_field(mut self, new_value: &str) -> PeopleGetCall<'a, S> {
        self._request_mask_include_field = Some(new_value.to_string());
        self
    }
    /// Required. A field mask to restrict which fields on the person are returned. Multiple fields can be specified by separating them with commas. Valid values are: * addresses * ageRanges * biographies * birthdays * calendarUrls * clientData * coverPhotos * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * metadata * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * photos * relations * sipAddresses * skills * urls * userDefined
    ///
    /// Sets the *person fields* query property to the given value.
    pub fn person_fields(mut self, new_value: &str) -> PeopleGetCall<'a, S> {
        self._person_fields = Some(new_value.to_string());
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleGetCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PeopleGetCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::ContactReadonly`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> PeopleGetCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Provides information about a list of specific people by specifying a list of requested resource names. Use `people/me` to indicate the authenticated user. The request returns a 400 error if 'personFields' is not specified.
///
/// A builder for the *getBatchGet* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().get_batch_get()
///              .add_sources("et")
///              .add_resource_names("sadipscing")
///              .request_mask_include_field("Stet")
///              .person_fields("dolor")
///              .doit().await;
/// # }
/// ```
pub struct PeopleGetBatchGetCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _sources: Vec<String>,
    _resource_names: Vec<String>,
    _request_mask_include_field: Option<String>,
    _person_fields: Option<String>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for PeopleGetBatchGetCall<'a, S> {}

impl<'a, S> PeopleGetBatchGetCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, GetPeopleResponse)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.people.getBatchGet",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(6 + self._additional_params.len());
        if self._sources.len() > 0 {
            for f in self._sources.iter() {
                params.push(("sources", f.to_string()));
            }
        }
        if self._resource_names.len() > 0 {
            for f in self._resource_names.iter() {
                params.push(("resourceNames", f.to_string()));
            }
        }
        if let Some(value) = self._request_mask_include_field {
            params.push(("requestMask.includeField", value.to_string()));
        }
        if let Some(value) = self._person_fields {
            params.push(("personFields", value.to_string()));
        }
        for &field in ["alt", "sources", "resourceNames", "requestMask.includeField", "personFields"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/people:batchGet";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::ContactReadonly.as_ref().to_string(), ());
        }


        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT and READ_SOURCE_TYPE_PROFILE if not set.
    ///
    /// Append the given value to the *sources* query property.
    /// Each appended value will retain its original ordering and be '/'-separated in the URL's parameters.
    pub fn add_sources(mut self, new_value: &str) -> PeopleGetBatchGetCall<'a, S> {
        self._sources.push(new_value.to_string());
        self
    }
    /// Required. The resource names of the people to provide information about. It's repeatable. The URL query parameter should be resourceNames=<name1>&resourceNames=<name2>&... - To get information about the authenticated user, specify `people/me`. - To get information about a google account, specify `people/{account_id}`. - To get information about a contact, specify the resource name that identifies the contact as returned by `people.connections.list`. There is a maximum of 200 resource names.
    ///
    /// Append the given value to the *resource names* query property.
    /// Each appended value will retain its original ordering and be '/'-separated in the URL's parameters.
    pub fn add_resource_names(mut self, new_value: &str) -> PeopleGetBatchGetCall<'a, S> {
        self._resource_names.push(new_value.to_string());
        self
    }
    /// Required. Comma-separated list of person fields to be included in the response. Each path should start with `person.`: for example, `person.names` or `person.photos`.
    ///
    /// Sets the *request mask.include field* query property to the given value.
    pub fn request_mask_include_field(mut self, new_value: &str) -> PeopleGetBatchGetCall<'a, S> {
        self._request_mask_include_field = Some(new_value.to_string());
        self
    }
    /// Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Valid values are: * addresses * ageRanges * biographies * birthdays * calendarUrls * clientData * coverPhotos * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * metadata * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * photos * relations * sipAddresses * skills * urls * userDefined
    ///
    /// Sets the *person fields* query property to the given value.
    pub fn person_fields(mut self, new_value: &str) -> PeopleGetBatchGetCall<'a, S> {
        self._person_fields = Some(new_value.to_string());
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleGetBatchGetCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PeopleGetBatchGetCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::ContactReadonly`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> PeopleGetBatchGetCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Provides a list of domain profiles and domain contacts in the authenticated user's domain directory. When the `sync_token` is specified, resources deleted since the last sync will be returned as a person with `PersonMetadata.deleted` set to true. When the `page_token` or `sync_token` is specified, all other request parameters must match the first call. Writes may have a propagation delay of several minutes for sync requests. Incremental syncs are not intended for read-after-write use cases. See example usage at [List the directory people that have changed](/people/v1/directory#list_the_directory_people_that_have_changed).
///
/// A builder for the *listDirectoryPeople* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().list_directory_people()
///              .sync_token("duo")
///              .add_sources("vero")
///              .request_sync_token(false)
///              .read_mask("invidunt")
///              .page_token("Stet")
///              .page_size(-76)
///              .add_merge_sources("elitr")
///              .doit().await;
/// # }
/// ```
pub struct PeopleListDirectoryPeopleCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _sync_token: Option<String>,
    _sources: Vec<String>,
    _request_sync_token: Option<bool>,
    _read_mask: Option<String>,
    _page_token: Option<String>,
    _page_size: Option<i32>,
    _merge_sources: Vec<String>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for PeopleListDirectoryPeopleCall<'a, S> {}

impl<'a, S> PeopleListDirectoryPeopleCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, ListDirectoryPeopleResponse)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.people.listDirectoryPeople",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(9 + self._additional_params.len());
        if let Some(value) = self._sync_token {
            params.push(("syncToken", value.to_string()));
        }
        if self._sources.len() > 0 {
            for f in self._sources.iter() {
                params.push(("sources", f.to_string()));
            }
        }
        if let Some(value) = self._request_sync_token {
            params.push(("requestSyncToken", value.to_string()));
        }
        if let Some(value) = self._read_mask {
            params.push(("readMask", value.to_string()));
        }
        if let Some(value) = self._page_token {
            params.push(("pageToken", value.to_string()));
        }
        if let Some(value) = self._page_size {
            params.push(("pageSize", value.to_string()));
        }
        if self._merge_sources.len() > 0 {
            for f in self._merge_sources.iter() {
                params.push(("mergeSources", f.to_string()));
            }
        }
        for &field in ["alt", "syncToken", "sources", "requestSyncToken", "readMask", "pageToken", "pageSize", "mergeSources"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/people:listDirectoryPeople";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::DirectoryReadonly.as_ref().to_string(), ());
        }


        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// Optional. A sync token, received from a previous response `next_sync_token` Provide this to retrieve only the resources changed since the last request. When syncing, all other parameters provided to `people.listDirectoryPeople` must match the first call that provided the sync token. More details about sync behavior at `people.listDirectoryPeople`.
    ///
    /// Sets the *sync token* query property to the given value.
    pub fn sync_token(mut self, new_value: &str) -> PeopleListDirectoryPeopleCall<'a, S> {
        self._sync_token = Some(new_value.to_string());
        self
    }
    /// Required. Directory sources to return.
    ///
    /// Append the given value to the *sources* query property.
    /// Each appended value will retain its original ordering and be '/'-separated in the URL's parameters.
    pub fn add_sources(mut self, new_value: &str) -> PeopleListDirectoryPeopleCall<'a, S> {
        self._sources.push(new_value.to_string());
        self
    }
    /// Optional. Whether the response should return `next_sync_token`. It can be used to get incremental changes since the last request by setting it on the request `sync_token`. More details about sync behavior at `people.listDirectoryPeople`.
    ///
    /// Sets the *request sync token* query property to the given value.
    pub fn request_sync_token(mut self, new_value: bool) -> PeopleListDirectoryPeopleCall<'a, S> {
        self._request_sync_token = Some(new_value);
        self
    }
    /// Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Valid values are: * addresses * ageRanges * biographies * birthdays * calendarUrls * clientData * coverPhotos * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * metadata * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * photos * relations * sipAddresses * skills * urls * userDefined
    ///
    /// Sets the *read mask* query property to the given value.
    pub fn read_mask(mut self, new_value: &str) -> PeopleListDirectoryPeopleCall<'a, S> {
        self._read_mask = Some(new_value.to_string());
        self
    }
    /// Optional. A page token, received from a previous response `next_page_token`. Provide this to retrieve the subsequent page. When paginating, all other parameters provided to `people.listDirectoryPeople` must match the first call that provided the page token.
    ///
    /// Sets the *page token* query property to the given value.
    pub fn page_token(mut self, new_value: &str) -> PeopleListDirectoryPeopleCall<'a, S> {
        self._page_token = Some(new_value.to_string());
        self
    }
    /// Optional. The number of people to include in the response. Valid values are between 1 and 1000, inclusive. Defaults to 100 if not set or set to 0.
    ///
    /// Sets the *page size* query property to the given value.
    pub fn page_size(mut self, new_value: i32) -> PeopleListDirectoryPeopleCall<'a, S> {
        self._page_size = Some(new_value);
        self
    }
    /// Optional. Additional data to merge into the directory sources if they are connected through verified join keys such as email addresses or phone numbers.
    ///
    /// Append the given value to the *merge sources* query property.
    /// Each appended value will retain its original ordering and be '/'-separated in the URL's parameters.
    pub fn add_merge_sources(mut self, new_value: &str) -> PeopleListDirectoryPeopleCall<'a, S> {
        self._merge_sources.push(new_value.to_string());
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleListDirectoryPeopleCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PeopleListDirectoryPeopleCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::DirectoryReadonly`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> PeopleListDirectoryPeopleCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Provides a list of contacts in the authenticated user's grouped contacts that matches the search query. The query matches on a contact's `names`, `nickNames`, `emailAddresses`, `phoneNumbers`, and `organizations` fields that are from the CONTACT source. **IMPORTANT**: Before searching, clients should send a warmup request with an empty query to update the cache. See https://developers.google.com/people/v1/contacts#search_the_users_contacts
///
/// A builder for the *searchContacts* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().search_contacts()
///              .add_sources("Lorem")
///              .read_mask("diam")
///              .query("no")
///              .page_size(-100)
///              .doit().await;
/// # }
/// ```
pub struct PeopleSearchContactCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _sources: Vec<String>,
    _read_mask: Option<String>,
    _query: Option<String>,
    _page_size: Option<i32>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for PeopleSearchContactCall<'a, S> {}

impl<'a, S> PeopleSearchContactCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, SearchResponse)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.people.searchContacts",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(6 + self._additional_params.len());
        if self._sources.len() > 0 {
            for f in self._sources.iter() {
                params.push(("sources", f.to_string()));
            }
        }
        if let Some(value) = self._read_mask {
            params.push(("readMask", value.to_string()));
        }
        if let Some(value) = self._query {
            params.push(("query", value.to_string()));
        }
        if let Some(value) = self._page_size {
            params.push(("pageSize", value.to_string()));
        }
        for &field in ["alt", "sources", "readMask", "query", "pageSize"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/people:searchContacts";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::ContactReadonly.as_ref().to_string(), ());
        }


        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT if not set.
    ///
    /// Append the given value to the *sources* query property.
    /// Each appended value will retain its original ordering and be '/'-separated in the URL's parameters.
    pub fn add_sources(mut self, new_value: &str) -> PeopleSearchContactCall<'a, S> {
        self._sources.push(new_value.to_string());
        self
    }
    /// Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Valid values are: * addresses * ageRanges * biographies * birthdays * calendarUrls * clientData * coverPhotos * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * metadata * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * photos * relations * sipAddresses * skills * urls * userDefined
    ///
    /// Sets the *read mask* query property to the given value.
    pub fn read_mask(mut self, new_value: &str) -> PeopleSearchContactCall<'a, S> {
        self._read_mask = Some(new_value.to_string());
        self
    }
    /// Required. The plain-text query for the request. The query is used to match prefix phrases of the fields on a person. For example, a person with name "foo name" matches queries such as "f", "fo", "foo", "foo n", "nam", etc., but not "oo n".
    ///
    /// Sets the *query* query property to the given value.
    pub fn query(mut self, new_value: &str) -> PeopleSearchContactCall<'a, S> {
        self._query = Some(new_value.to_string());
        self
    }
    /// Optional. The number of results to return. Defaults to 10 if field is not set, or set to 0. Values greater than 30 will be capped to 30.
    ///
    /// Sets the *page size* query property to the given value.
    pub fn page_size(mut self, new_value: i32) -> PeopleSearchContactCall<'a, S> {
        self._page_size = Some(new_value);
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleSearchContactCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PeopleSearchContactCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::ContactReadonly`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> PeopleSearchContactCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Provides a list of domain profiles and domain contacts in the authenticated user's domain directory that match the search query.
///
/// A builder for the *searchDirectoryPeople* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().search_directory_people()
///              .add_sources("accusam")
///              .read_mask("takimata")
///              .query("consetetur")
///              .page_token("voluptua.")
///              .page_size(-72)
///              .add_merge_sources("erat")
///              .doit().await;
/// # }
/// ```
pub struct PeopleSearchDirectoryPeopleCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _sources: Vec<String>,
    _read_mask: Option<String>,
    _query: Option<String>,
    _page_token: Option<String>,
    _page_size: Option<i32>,
    _merge_sources: Vec<String>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for PeopleSearchDirectoryPeopleCall<'a, S> {}

impl<'a, S> PeopleSearchDirectoryPeopleCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, SearchDirectoryPeopleResponse)> {
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.people.searchDirectoryPeople",
                               http_method: hyper::Method::GET });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(8 + self._additional_params.len());
        if self._sources.len() > 0 {
            for f in self._sources.iter() {
                params.push(("sources", f.to_string()));
            }
        }
        if let Some(value) = self._read_mask {
            params.push(("readMask", value.to_string()));
        }
        if let Some(value) = self._query {
            params.push(("query", value.to_string()));
        }
        if let Some(value) = self._page_token {
            params.push(("pageToken", value.to_string()));
        }
        if let Some(value) = self._page_size {
            params.push(("pageSize", value.to_string()));
        }
        if self._merge_sources.len() > 0 {
            for f in self._merge_sources.iter() {
                params.push(("mergeSources", f.to_string()));
            }
        }
        for &field in ["alt", "sources", "readMask", "query", "pageToken", "pageSize", "mergeSources"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/people:searchDirectoryPeople";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::DirectoryReadonly.as_ref().to_string(), ());
        }


        let url = url::Url::parse_with_params(&url, params).unwrap();



        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::GET).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .body(hyper::body::Body::empty());

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    /// Required. Directory sources to return.
    ///
    /// Append the given value to the *sources* query property.
    /// Each appended value will retain its original ordering and be '/'-separated in the URL's parameters.
    pub fn add_sources(mut self, new_value: &str) -> PeopleSearchDirectoryPeopleCall<'a, S> {
        self._sources.push(new_value.to_string());
        self
    }
    /// Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Valid values are: * addresses * ageRanges * biographies * birthdays * calendarUrls * clientData * coverPhotos * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * metadata * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * photos * relations * sipAddresses * skills * urls * userDefined
    ///
    /// Sets the *read mask* query property to the given value.
    pub fn read_mask(mut self, new_value: &str) -> PeopleSearchDirectoryPeopleCall<'a, S> {
        self._read_mask = Some(new_value.to_string());
        self
    }
    /// Required. Prefix query that matches fields in the person. Does NOT use the read_mask for determining what fields to match.
    ///
    /// Sets the *query* query property to the given value.
    pub fn query(mut self, new_value: &str) -> PeopleSearchDirectoryPeopleCall<'a, S> {
        self._query = Some(new_value.to_string());
        self
    }
    /// Optional. A page token, received from a previous response `next_page_token`. Provide this to retrieve the subsequent page. When paginating, all other parameters provided to `SearchDirectoryPeople` must match the first call that provided the page token.
    ///
    /// Sets the *page token* query property to the given value.
    pub fn page_token(mut self, new_value: &str) -> PeopleSearchDirectoryPeopleCall<'a, S> {
        self._page_token = Some(new_value.to_string());
        self
    }
    /// Optional. The number of people to include in the response. Valid values are between 1 and 500, inclusive. Defaults to 100 if not set or set to 0.
    ///
    /// Sets the *page size* query property to the given value.
    pub fn page_size(mut self, new_value: i32) -> PeopleSearchDirectoryPeopleCall<'a, S> {
        self._page_size = Some(new_value);
        self
    }
    /// Optional. Additional data to merge into the directory sources if they are connected through verified join keys such as email addresses or phone numbers.
    ///
    /// Append the given value to the *merge sources* query property.
    /// Each appended value will retain its original ordering and be '/'-separated in the URL's parameters.
    pub fn add_merge_sources(mut self, new_value: &str) -> PeopleSearchDirectoryPeopleCall<'a, S> {
        self._merge_sources.push(new_value.to_string());
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleSearchDirectoryPeopleCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PeopleSearchDirectoryPeopleCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::DirectoryReadonly`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> PeopleSearchDirectoryPeopleCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Update contact data for an existing contact person. Any non-contact data will not be modified. Any non-contact data in the person to update will be ignored. All fields specified in the `update_mask` will be replaced. The server returns a 400 error if `person.metadata.sources` is not specified for the contact to be updated or if there is no contact source. The server returns a 400 error with reason `"failedPrecondition"` if `person.metadata.sources.etag` is different than the contact's etag, which indicates the contact has changed since its data was read. Clients should get the latest person and merge their updates into the latest person. The server returns a 400 error if `memberships` are being updated and there are no contact group memberships specified on the person. The server returns a 400 error if more than one field is specified on a field that is a singleton for contact sources: * biographies * birthdays * genders * names Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
///
/// A builder for the *updateContact* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// use people1::api::Person;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // As the method needs a request, you would usually fill it with the desired information
/// // into the respective structure. Some of the parts shown here might not be applicable !
/// // Values shown here are possibly random and not representative !
/// let mut req = Person::default();
/// 
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().update_contact(req, "resourceName")
///              .update_person_fields("amet.")
///              .add_sources("sed")
///              .person_fields("takimata")
///              .doit().await;
/// # }
/// ```
pub struct PeopleUpdateContactCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _request: Person,
    _resource_name: String,
    _update_person_fields: Option<String>,
    _sources: Vec<String>,
    _person_fields: Option<String>,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for PeopleUpdateContactCall<'a, S> {}

impl<'a, S> PeopleUpdateContactCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, Person)> {
        use url::percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.people.updateContact",
                               http_method: hyper::Method::PATCH });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(7 + self._additional_params.len());
        params.push(("resourceName", self._resource_name.to_string()));
        if let Some(value) = self._update_person_fields {
            params.push(("updatePersonFields", value.to_string()));
        }
        if self._sources.len() > 0 {
            for f in self._sources.iter() {
                params.push(("sources", f.to_string()));
            }
        }
        if let Some(value) = self._person_fields {
            params.push(("personFields", value.to_string()));
        }
        for &field in ["alt", "resourceName", "updatePersonFields", "sources", "personFields"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/{+resourceName}:updateContact";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::Contact.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{+resourceName}", "resourceName")].iter() {
            let mut replace_with = String::new();
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = value.to_string();
                    break;
                }
            }
            if find_this.as_bytes()[1] == '+' as u8 {
                replace_with = percent_encode(replace_with.as_bytes(), DEFAULT_ENCODE_SET).to_string();
            }
            url = url.replace(find_this, &replace_with);
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["resourceName"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();

        let mut json_mime_type: mime::Mime = "application/json".parse().unwrap();
        let mut request_value_reader =
            {
                let mut value = json::value::to_value(&self._request).expect("serde to work");
                client::remove_json_null_values(&mut value);
                let mut dst = io::Cursor::new(Vec::with_capacity(128));
                json::to_writer(&mut dst, &value).unwrap();
                dst
            };
        let request_size = request_value_reader.seek(io::SeekFrom::End(0)).unwrap();
        request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();


        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::PATCH).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .header(CONTENT_TYPE, format!("{}", json_mime_type.to_string()))
                        .header(CONTENT_LENGTH, request_size as u64)
                        .body(hyper::body::Body::from(request_value_reader.get_ref().clone()));

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    ///
    /// Sets the *request* property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn request(mut self, new_value: Person) -> PeopleUpdateContactCall<'a, S> {
        self._request = new_value;
        self
    }
    /// The resource name for the person, assigned by the server. An ASCII string with a max length of 27 characters, in the form of `people/{person_id}`.
    ///
    /// Sets the *resource name* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn resource_name(mut self, new_value: &str) -> PeopleUpdateContactCall<'a, S> {
        self._resource_name = new_value.to_string();
        self
    }
    /// Required. A field mask to restrict which fields on the person are updated. Multiple fields can be specified by separating them with commas. All updated fields will be replaced. Valid values are: * addresses * biographies * birthdays * calendarUrls * clientData * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * relations * sipAddresses * urls * userDefined
    ///
    /// Sets the *update person fields* query property to the given value.
    pub fn update_person_fields(mut self, new_value: &str) -> PeopleUpdateContactCall<'a, S> {
        self._update_person_fields = Some(new_value.to_string());
        self
    }
    /// Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT and READ_SOURCE_TYPE_PROFILE if not set.
    ///
    /// Append the given value to the *sources* query property.
    /// Each appended value will retain its original ordering and be '/'-separated in the URL's parameters.
    pub fn add_sources(mut self, new_value: &str) -> PeopleUpdateContactCall<'a, S> {
        self._sources.push(new_value.to_string());
        self
    }
    /// Optional. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Defaults to all fields if not set. Valid values are: * addresses * ageRanges * biographies * birthdays * calendarUrls * clientData * coverPhotos * emailAddresses * events * externalIds * genders * imClients * interests * locales * locations * memberships * metadata * miscKeywords * names * nicknames * occupations * organizations * phoneNumbers * photos * relations * sipAddresses * skills * urls * userDefined
    ///
    /// Sets the *person fields* query property to the given value.
    pub fn person_fields(mut self, new_value: &str) -> PeopleUpdateContactCall<'a, S> {
        self._person_fields = Some(new_value.to_string());
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleUpdateContactCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PeopleUpdateContactCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::Contact`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> PeopleUpdateContactCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


/// Update a contact's photo. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.
///
/// A builder for the *updateContactPhoto* method supported by a *people* resource.
/// It is not used directly, but through a `PeopleMethods` instance.
///
/// # Example
///
/// Instantiate a resource method builder
///
/// ```test_harness,no_run
/// # extern crate hyper;
/// # extern crate hyper_rustls;
/// # extern crate google_people1 as people1;
/// use people1::api::UpdateContactPhotoRequest;
/// # async fn dox() {
/// # use std::default::Default;
/// # use people1::{PeopleService, oauth2, hyper, hyper_rustls};
/// 
/// # let secret: oauth2::ApplicationSecret = Default::default();
/// # let auth = oauth2::InstalledFlowAuthenticator::builder(
/// #         secret,
/// #         oauth2::InstalledFlowReturnMethod::HTTPRedirect,
/// #     ).build().await.unwrap();
/// # let mut hub = PeopleService::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
/// // As the method needs a request, you would usually fill it with the desired information
/// // into the respective structure. Some of the parts shown here might not be applicable !
/// // Values shown here are possibly random and not representative !
/// let mut req = UpdateContactPhotoRequest::default();
/// 
/// // You can configure optional parameters by calling the respective setters at will, and
/// // execute the final call using `doit()`.
/// // Values shown here are possibly random and not representative !
/// let result = hub.people().update_contact_photo(req, "resourceName")
///              .doit().await;
/// # }
/// ```
pub struct PeopleUpdateContactPhotoCall<'a, S>
    where S: 'a {

    hub: &'a PeopleService<S>,
    _request: UpdateContactPhotoRequest,
    _resource_name: String,
    _delegate: Option<&'a mut dyn client::Delegate>,
    _additional_params: HashMap<String, String>,
    _scopes: BTreeMap<String, ()>
}

impl<'a, S> client::CallBuilder for PeopleUpdateContactPhotoCall<'a, S> {}

impl<'a, S> PeopleUpdateContactPhotoCall<'a, S>
where
    S: tower_service::Service<Uri> + Clone + Send + Sync + 'static,
    S::Response: hyper::client::connect::Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S::Future: Send + Unpin + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
{


    /// Perform the operation you have build so far.
    pub async fn doit(mut self) -> client::Result<(hyper::Response<hyper::body::Body>, UpdateContactPhotoResponse)> {
        use url::percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};
        use std::io::{Read, Seek};
        use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH, AUTHORIZATION, USER_AGENT, LOCATION};
        use client::ToParts;
        let mut dd = client::DefaultDelegate;
        let mut dlg: &mut dyn client::Delegate = match self._delegate {
            Some(d) => d,
            None => &mut dd
        };
        dlg.begin(client::MethodInfo { id: "people.people.updateContactPhoto",
                               http_method: hyper::Method::PATCH });
        let mut params: Vec<(&str, String)> = Vec::with_capacity(4 + self._additional_params.len());
        params.push(("resourceName", self._resource_name.to_string()));
        for &field in ["alt", "resourceName"].iter() {
            if self._additional_params.contains_key(field) {
                dlg.finished(false);
                return Err(client::Error::FieldClash(field));
            }
        }
        for (name, value) in self._additional_params.iter() {
            params.push((&name, value.clone()));
        }

        params.push(("alt", "json".to_string()));

        let mut url = self.hub._base_url.clone() + "v1/{+resourceName}:updateContactPhoto";
        if self._scopes.len() == 0 {
            self._scopes.insert(Scope::Contact.as_ref().to_string(), ());
        }

        for &(find_this, param_name) in [("{+resourceName}", "resourceName")].iter() {
            let mut replace_with = String::new();
            for &(name, ref value) in params.iter() {
                if name == param_name {
                    replace_with = value.to_string();
                    break;
                }
            }
            if find_this.as_bytes()[1] == '+' as u8 {
                replace_with = percent_encode(replace_with.as_bytes(), DEFAULT_ENCODE_SET).to_string();
            }
            url = url.replace(find_this, &replace_with);
        }
        {
            let mut indices_for_removal: Vec<usize> = Vec::with_capacity(1);
            for param_name in ["resourceName"].iter() {
                if let Some(index) = params.iter().position(|t| &t.0 == param_name) {
                    indices_for_removal.push(index);
                }
            }
            for &index in indices_for_removal.iter() {
                params.remove(index);
            }
        }

        let url = url::Url::parse_with_params(&url, params).unwrap();

        let mut json_mime_type: mime::Mime = "application/json".parse().unwrap();
        let mut request_value_reader =
            {
                let mut value = json::value::to_value(&self._request).expect("serde to work");
                client::remove_json_null_values(&mut value);
                let mut dst = io::Cursor::new(Vec::with_capacity(128));
                json::to_writer(&mut dst, &value).unwrap();
                dst
            };
        let request_size = request_value_reader.seek(io::SeekFrom::End(0)).unwrap();
        request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();


        loop {
            let token = match self.hub.auth.token(&self._scopes.keys().collect::<Vec<_>>()[..]).await {
                Ok(token) => token.clone(),
                Err(err) => {
                    match  dlg.token(&err) {
                        Some(token) => token,
                        None => {
                            dlg.finished(false);
                            return Err(client::Error::MissingToken(err))
                        }
                    }
                }
            };
            request_value_reader.seek(io::SeekFrom::Start(0)).unwrap();
            let mut req_result = {
                let client = &self.hub.client;
                dlg.pre_request();
                let mut req_builder = hyper::Request::builder().method(hyper::Method::PATCH).uri(url.clone().into_string())
                        .header(USER_AGENT, self.hub._user_agent.clone())                            .header(AUTHORIZATION, format!("Bearer {}", token.as_str()));


                        let request = req_builder
                        .header(CONTENT_TYPE, format!("{}", json_mime_type.to_string()))
                        .header(CONTENT_LENGTH, request_size as u64)
                        .body(hyper::body::Body::from(request_value_reader.get_ref().clone()));

                client.request(request.unwrap()).await
                
            };

            match req_result {
                Err(err) => {
                    if let client::Retry::After(d) = dlg.http_error(&err) {
                        sleep(d);
                        continue;
                    }
                    dlg.finished(false);
                    return Err(client::Error::HttpError(err))
                }
                Ok(mut res) => {
                    if !res.status().is_success() {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;
                        let (parts, _) = res.into_parts();
                        let body = hyper::Body::from(res_body_string.clone());
                        let restored_response = hyper::Response::from_parts(parts, body);

                        let server_response = json::from_str::<serde_json::Value>(&res_body_string).ok();

                        if let client::Retry::After(d) = dlg.http_failure(&restored_response, server_response.clone()) {
                            sleep(d);
                            continue;
                        }

                        dlg.finished(false);

                        return match server_response {
                            Some(error_value) => Err(client::Error::BadRequest(error_value)),
                            None => Err(client::Error::Failure(restored_response)),
                        }
                    }
                    let result_value = {
                        let res_body_string = client::get_body_as_string(res.body_mut()).await;

                        match json::from_str(&res_body_string) {
                            Ok(decoded) => (res, decoded),
                            Err(err) => {
                                dlg.response_json_decode_error(&res_body_string, &err);
                                return Err(client::Error::JsonDecodeError(res_body_string, err));
                            }
                        }
                    };

                    dlg.finished(true);
                    return Ok(result_value)
                }
            }
        }
    }


    ///
    /// Sets the *request* property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn request(mut self, new_value: UpdateContactPhotoRequest) -> PeopleUpdateContactPhotoCall<'a, S> {
        self._request = new_value;
        self
    }
    /// Required. Person resource name
    ///
    /// Sets the *resource name* path property to the given value.
    ///
    /// Even though the property as already been set when instantiating this call,
    /// we provide this method for API completeness.
    pub fn resource_name(mut self, new_value: &str) -> PeopleUpdateContactPhotoCall<'a, S> {
        self._resource_name = new_value.to_string();
        self
    }
    /// The delegate implementation is consulted whenever there is an intermediate result, or if something goes wrong
    /// while executing the actual API request.
    /// 
    /// It should be used to handle progress information, and to implement a certain level of resilience.
    ///
    /// Sets the *delegate* property to the given value.
    pub fn delegate(mut self, new_value: &'a mut dyn client::Delegate) -> PeopleUpdateContactPhotoCall<'a, S> {
        self._delegate = Some(new_value);
        self
    }

    /// Set any additional parameter of the query string used in the request.
    /// It should be used to set parameters which are not yet available through their own
    /// setters.
    ///
    /// Please note that this method must not be used to set any of the known parameters
    /// which have their own setter method. If done anyway, the request will fail.
    ///
    /// # Additional Parameters
    ///
    /// * *$.xgafv* (query-string) - V1 error format.
    /// * *access_token* (query-string) - OAuth access token.
    /// * *alt* (query-string) - Data format for response.
    /// * *callback* (query-string) - JSONP
    /// * *fields* (query-string) - Selector specifying which fields to include in a partial response.
    /// * *key* (query-string) - API key. Your API key identifies your project and provides you with API access, quota, and reports. Required unless you provide an OAuth 2.0 token.
    /// * *oauth_token* (query-string) - OAuth 2.0 token for the current user.
    /// * *prettyPrint* (query-boolean) - Returns response with indentations and line breaks.
    /// * *quotaUser* (query-string) - Available to use for quota purposes for server-side applications. Can be any arbitrary string assigned to a user, but should not exceed 40 characters.
    /// * *uploadType* (query-string) - Legacy upload protocol for media (e.g. "media", "multipart").
    /// * *upload_protocol* (query-string) - Upload protocol for media (e.g. "raw", "multipart").
    pub fn param<T>(mut self, name: T, value: T) -> PeopleUpdateContactPhotoCall<'a, S>
                                                        where T: AsRef<str> {
        self._additional_params.insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Identifies the authorization scope for the method you are building.
    ///
    /// Use this method to actively specify which scope should be used, instead the default `Scope` variant
    /// `Scope::Contact`.
    ///
    /// The `scope` will be added to a set of scopes. This is important as one can maintain access
    /// tokens for more than one scope.
    /// If `None` is specified, then all scopes will be removed and no default scope will be used either.
    /// In that case, you have to specify your API-key using the `key` parameter (see the `param()`
    /// function for details).
    ///
    /// Usually there is more than one suitable scope to authorize an operation, some of which may
    /// encompass more rights than others. For example, for listing resources, a *read-only* scope will be
    /// sufficient, a read-write scope will do as well.
    pub fn add_scope<T, St>(mut self, scope: T) -> PeopleUpdateContactPhotoCall<'a, S>
                                                        where T: Into<Option<St>>,
                                                              St: AsRef<str> {
        match scope.into() {
          Some(scope) => self._scopes.insert(scope.as_ref().to_string(), ()),
          None => None,
        };
        self
    }
}


