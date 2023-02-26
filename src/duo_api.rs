//! Exports [`DuoApi`] for all API related functionality.

use fake::{
    faker::internet::en::{FreeEmail, Password, UserAgent},
    Fake,
};
use reqwest::{
    blocking::{Client, ClientBuilder, Response},
    header::COOKIE,
    redirect::Policy,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The API used to create and patch users. This specific API version is the
/// only one that supports this strategy.
pub const BASE_USERS_URL: &str = "https://www.duolingo.com/2017-06-30/users";
pub const RECAPTCHA_SIGNAL_SIT_KEY: &str = "6LcLOdsjAAAAAFfwGusLLnnn492SOGhsCh-uEAvI";

/// Data used to bypass the reCAPTCHA check.
#[derive(Serialize)]
pub struct UserCreationSignal {
    site_key: String,
    token: String,
    vendor: u8,
}

impl Default for UserCreationSignal {
    /// Creates a new [`UserCreationSignal`] with the default values.
    fn default() -> Self {
        Self {
            site_key: RECAPTCHA_SIGNAL_SIT_KEY.to_string(),
            // At the moment, Duo doesn't check this value.
            token: String::from("-"),
            vendor: 2,
        }
    }
}

/// The data sent to the API to create a user. `timezone` and `from_language`
/// are dummy values, but `invite_code` is the provided code and `distinct_id`
/// is a UUIDv4.
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserCreationData {
    timezone: String,
    from_language: String,
    invite_code: String,
    distinct_id: String,
    signal: UserCreationSignal,
}

/// The data sent to the API to create credentials. All 3 of these fields are
/// dummy values.
#[derive(Serialize, Default)]
pub struct UserCredentialsData {
    email: String,
    password: String,
    age: String,
    signal: UserCreationSignal,
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

impl From<Response> for AccountData {
    /// Finds the JWT token from the request to be reused on the next request as
    /// a form of authentication, and then deserializes the response as JSON to
    /// retrieve the user id.
    fn from(res: Response) -> Self {
        let token = res.headers()["jwt"]
            .to_str()
            .expect("JWT token was not found in the account creation response headers")
            .to_string();

        let id = res
            .json::<UserCreationResponse>()
            .expect("Failed to parse user creation response")
            .id;

        Self { id, token }
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
    /// Validates a referral code. If a link is given, the base will be
    /// stripped, leaving (hopefully) just the code. It must be a 26-length string of
    /// ascii_alphanumeric characters. After validated, the string will be
    /// converted to uppercase.
    pub fn parse_code(code: &str) -> Result<String, String> {
        let parsed_code = code.replace("https://invite.duolingo.com/", "");
        if parsed_code.len() == 26 && parsed_code.chars().all(|char| char.is_ascii_alphanumeric()) {
            Ok(parsed_code.to_uppercase())
        } else {
            Err(String::from("Invalid referral code/link"))
        }
    }

    /// Creates a new user via the provided referral code (see
    /// [`UserCreationData`]), and constructs a [`AccountData`] from it.
    pub fn create_account(&self, code: &str) -> AccountData {
        let creation_data = UserCreationData {
            timezone: String::from("America/Montreal"),
            from_language: String::from("en"),
            invite_code: code.to_string(),
            distinct_id: Uuid::new_v4().to_string(),
            ..Default::default()
        };

        let res = self
            .client
            .post(format!("{BASE_USERS_URL}?fields=id"))
            .json(&creation_data)
            .send()
            .unwrap()
            .error_for_status()
            .expect("Failed to create account");

        res.into()
    }

    /// Creates credentials for the user (see [`UserCredentialsData`]) from [`AccountData`].
    pub fn create_credentials(&self, data: &AccountData) {
        let user_data = UserCredentialsData {
            age: String::from("5"),
            email: FreeEmail().fake(),
            password: Password(15..16).fake(),
            ..Default::default()
        };

        self.client
            .patch(format!("{BASE_USERS_URL}/{}?fields=email,identifier,name,username", data.id))
            .header(COOKIE, format!("jwt_token={}", data.token))
            .json(&user_data)
            .send()
            .unwrap()
            .error_for_status()
            .expect("Failed to create credentials");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_code() {
        let code = "A".repeat(26);
        assert_eq!(DuoApi::parse_code(&code), Ok(code.to_string()));
    }

    #[test]
    fn valid_link() {
        let code = "A".repeat(26);
        let link = format!("https://invite.duolingo.com/{code}");

        assert_eq!(DuoApi::parse_code(&link), Ok(code));
    }

    #[test]
    fn undercase_code() {
        let code = "a".repeat(26);
        assert_eq!(DuoApi::parse_code(&code), Ok(code.to_uppercase()));
    }

    #[test]
    fn incorrect_length_code() {
        let short_code = "A".repeat(25);
        let long_code = "A".repeat(27);

        assert!(DuoApi::parse_code(&short_code).is_err());
        assert!(DuoApi::parse_code(&long_code).is_err());
    }

    #[test]
    fn incorrect_characters_code() {
        let code = "_".repeat(26);
        assert!(DuoApi::parse_code(code.as_str()).is_err());
    }
}
