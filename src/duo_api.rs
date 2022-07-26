//! Exports [`DuoApi`] for all API related functionality.

use fake::{
    faker::internet::en::{FreeEmail, Password},
    Fake,
};
use reqwest::{redirect::Policy, Client, ClientBuilder, Error, Response};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The API used to create and patch users. This specific API version is the
/// only one that supports this strategy.
pub const BASE_USERS_URL: &str = "https://www.duolingo.com/2017-06-30/users";

/// User-Agent header used for all requests.
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2704.79 Safari/537.36 Edge/14.14393";

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
    pub async fn from(res: Response) -> Result<Self, Error> {
        let token = res.headers()["jwt"].to_str().unwrap().to_owned();
        let id = res.json::<UserCreationResponse>().await?.id;
        Ok(Self { id, token })
    }
}

/// Stores the [`reqwest::Client`] used to make requests to the API and all
/// functionality relevant to sending requests to the API.
#[derive(Clone)]
pub struct DuoApi {
    client: Client,
}

impl DuoApi {
    /// Creates a new API client with a reusable User-Agent.
    pub fn new() -> Self {
        Self {
            client: ClientBuilder::new()
                // The user agent will make the request look less like a bot's.
                .user_agent(USER_AGENT)
                // [`DuoApi::get_user_id`] makes a request that will try to redirect the user, which we don't want.
                .redirect(Policy::none())
                .build()
                .unwrap(),
        }
    }

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
    pub async fn create_account(&self, code: &str) -> Result<AccountData, Error> {
        let uuid = Uuid::new_v4().to_string();

        let creation_data = UserCreationData {
            timezone: "America/Montreal".into(),
            from_language: "en".into(),
            invite_code: code.into(),
            distinct_id: uuid.into(),
        };

        let res = self
            .client
            .post(format!("{BASE_USERS_URL}?fields=id"))
            .json(&creation_data)
            .send()
            .await?;

        Ok(AccountData::from(res).await?)
    }

    /// Creates credentials for the user (see [`UserCredentialsData`]) from [`AccountData`].
    pub async fn create_credentials(&self, data: &AccountData) -> Result<(), Error> {
        let user_data = UserCredentialsData {
            age: "5".into(),
            email: FreeEmail().fake(),
            password: Password(15..16).fake(),
        };

        self.client
            .patch(format!("{BASE_USERS_URL}/{}?fields=none", data.id))
            .header("Cookie", format!("jwt_token={}", data.token))
            .json(&user_data)
            .send()
            .await?;

        Ok(())
    }
}
