//! Exports [`DuoApi`] for all API related functionality.

use std::collections::HashMap;

use fake::{
    faker::internet::en::{FreeEmail, Password, UserAgent},
    Fake,
};
use reqwest::{
    blocking::{Client, ClientBuilder, Response},
    header::COOKIE,
    redirect::Policy,
    Error, Url,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The API used to create and patch users. This specific API version is the
/// only one that supports this strategy.
pub const BASE_USERS_URL: &str = "https://www.duolingo.com/2017-06-30/users";

/// The API used to request data on how many more free weeks of Plus they can
/// claim and to actually accept a duolingo invite, which is used to find the
/// original user's ID.
pub const BASE_INVITE_URL: &str = "https://invite.duolingo.com";

/// The data sent to the API to create a user. `timezone` and `from_language`
/// are dummy values, but `invite_code` is the provided code and `distinct_id`
/// is a UUIDv4.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserCreationData {
    timezone: String,
    from_language: String,
    invite_code: String,
    distinct_id: String,
}

/// The data sent to the API to create credentials. All 3 of these fields are
/// dummy values.
#[derive(Serialize)]
pub struct UserCredentialsData {
    email: String,
    password: String,
    age: String,
}

/// The data returned from the API after creating the user with
/// [`UserCreationData`]. `id` is the unique numerical identifier for the user
/// that will be appended to the [`BASE_USERS_URL`] for all subsequent requests.
#[derive(Deserialize)]
pub struct UserCreationResponse {
    id: u32,
}

/// The data returned from the API representing the additional number of free
/// weeks of Plus available to the user.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteStatusResponse {
    num_weeks_available: u8,
}

/// All relevant data from creating the user needed to create credentials. This
/// includes the user ID and the JWT token returned from the API when creating
/// the user.
pub struct AccountData {
    id: u32,
    token: String,
}

impl AccountData {
    /// Finds the JWT token from the request to be reused on the next request as
    /// a form of authentication, and then deserializes the response as JSON to
    /// retrieve the user id.
    pub fn from(res: Response) -> Result<Self, Error> {
        let token = res.headers()["jwt"].to_str().unwrap().to_owned();
        let id = res.json::<UserCreationResponse>()?.id;
        Ok(Self { id, token })
    }
}

/// Stores the [`reqwest::Client`] used to make requests to the API and all
/// functionality relevant to sending requests to the API.
pub struct DuoApi {
    client: Client,
}

impl Default for DuoApi {
    /// Creates a new API client with a reusable User-Agent.
    fn default() -> Self {
        Self {
            client: ClientBuilder::new()
                // The user agent will make the request look less like a bot's.
                .user_agent(UserAgent().fake::<&str>())
                // [`DuoApi::get_user_id`] makes a request that will try to redirect the user, which we don't want.
                .redirect(Policy::none())
                .build()
                .unwrap(),
        }
    }
}

impl DuoApi {
    /// Validates a referral code. It must be a 26-length string of
    /// ascii_alphanumeric characters. After validated, the string will be
    /// converted to uppercase.
    pub fn is_valid_code(code: &str) -> Result<String, String> {
        if code.len() == 26 && code.chars().all(|c| c.is_ascii_alphanumeric()) {
            Ok(code.to_uppercase())
        } else {
            Err("Invalid referral code".to_owned())
        }
    }

    /// Creates a new user via the provided referral code (see
    /// [`UserCreationData`]), and constructs a [`AccountData`] from it.
    pub fn create_account(&self, code: &str) -> Result<AccountData, Error> {
        let creation_data = UserCreationData {
            timezone: String::from("America/Montreal"),
            from_language: String::from("en"),
            invite_code: code.to_owned(),
            distinct_id: Uuid::new_v4().to_string(),
        };

        let res = self
            .client
            .post(format!("{BASE_USERS_URL}?fields=id"))
            .json(&creation_data)
            .send()?
            .error_for_status()?;

        AccountData::from(res)
    }

    /// Creates credentials for the user (see [`UserCredentialsData`]) from [`AccountData`].
    pub fn create_credentials(&self, data: &AccountData) -> Result<(), Error> {
        let user_data = UserCredentialsData {
            age: "5".into(),
            email: FreeEmail().fake(),
            password: Password(15..16).fake(),
        };

        self.client
            .patch(format!("{BASE_USERS_URL}/{}?fields=none", data.id))
            .header(COOKIE, format!("jwt_token={}", data.token))
            .json(&user_data)
            .send()?
            .error_for_status()?;

        Ok(())
    }

    /// Gets the ID of the original user from the referral code by requesting
    /// the referral link (based from [`BASE_INVITE_URL`]), which will redirect
    /// you to the main duolingo domain with metadata such as `inviter_id` in
    /// the query string. It will then redirect again, which we don't want, so
    /// we instead disable all redirects and parse the URL we want from the
    /// location header.
    pub fn get_user_id(&self, code: &str) -> Result<String, Error> {
        let res = self
            .client
            .get(format!("{BASE_INVITE_URL}/{code}"))
            .send()?;

        let location_header = res.headers()["location"]
            .to_str()
            .unwrap()
            // The duolingo URL we want is actually an encoded query string
            // value, so we apply basic decoding to get `inviter_id` into the
            // pairs detected by [`reqwest::Url::query_pairs`].
            .replace("%26", "&")
            .replace("%3D", "=");

        let redirect_url = Url::parse(&location_header).unwrap();
        let query_params: HashMap<_, _> = redirect_url.query_pairs().into_owned().collect();

        Ok(query_params["inviter_id"].to_owned())
    }

    /// Finds out the additional number of free weeks of Plus available to the user.
    pub fn check_invites_left(&self, data: &AccountData, code: &str) -> Result<u8, Error> {
        let url = format!(
            "{BASE_INVITE_URL}/user/{}/tiered-rewards/status",
            self.get_user_id(code)?
        );

        let res: InviteStatusResponse = self
            .client
            .get(url)
            .header(COOKIE, format!("jwt_token={}", data.token))
            .send()?
            .error_for_status()?
            .json()?;

        Ok(res.num_weeks_available)
    }
}
