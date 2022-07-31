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
    Url,
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
    pub fn from(res: Response) -> Self {
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
        let parsed_code = code.replace(&format!("{BASE_INVITE_URL}/"), "");
        if parsed_code.len() == 26 && parsed_code.chars().all(|c| c.is_ascii_alphanumeric()) {
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
        };

        let res = self
            .client
            .post(format!("{BASE_USERS_URL}?fields=id"))
            .json(&creation_data)
            .send()
            .unwrap()
            .error_for_status()
            .expect("Failed to create account");

        AccountData::from(res)
    }

    /// Creates credentials for the user (see [`UserCredentialsData`]) from [`AccountData`].
    pub fn create_credentials(&self, data: &AccountData) {
        let user_data = UserCredentialsData {
            age: String::from("5"),
            email: FreeEmail().fake(),
            password: Password(15..16).fake(),
        };

        self.client
            .patch(format!("{BASE_USERS_URL}/{}?fields=none", data.id))
            .header(COOKIE, format!("jwt_token={}", data.token))
            .json(&user_data)
            .send()
            .unwrap()
            .error_for_status()
            .expect("Failed to create credentials");
    }

    /// Gets the ID of the original user from the referral code by requesting
    /// the referral link (based from [`BASE_INVITE_URL`]), which will redirect
    /// you to the main duolingo domain with metadata such as `inviter_id` in
    /// the query string. It will then redirect again, which we don't want, so
    /// we instead disable all redirects and parse the URL we want from the
    /// location header.
    pub fn get_user_id(&self, code: &str) -> String {
        let res = self
            .client
            .get(format!("{BASE_INVITE_URL}/{code}"))
            .send()
            .unwrap()
            .error_for_status()
            .expect("Failed to request invite link");

        let location_header = res.headers()["location"]
            .to_str()
            .expect("Failed to parse location header");

        DuoApi::resolve_inviter_id(location_header)
    }

    /// Parse inviter ID from the "location" header of the response in [`DuoApi::get_user_id`].
    pub fn resolve_inviter_id(location: &str) -> String {
        let location_header = location
            // The duolingo URL we want is actually an encoded query string
            // value, so we apply basic decoding to get `inviter_id` into the
            // pairs detected by [`reqwest::Url::query_pairs`].
            .replace("%26", "&")
            .replace("%3D", "=");

        let redirect_url = Url::parse(&location_header) //
            .expect("Failed to parse location header URL");

        let query_params: HashMap<_, _> = redirect_url.query_pairs().into_owned().collect();

        query_params["inviter_id"].to_owned()
    }

    /// Finds out the additional number of free weeks of Plus available to the user.
    pub fn check_invites_left(&self, data: &AccountData, code: &str) -> u8 {
        let url = format!(
            "{BASE_INVITE_URL}/user/{}/tiered-rewards/status",
            self.get_user_id(code)
        );

        let res: InviteStatusResponse = self
            .client
            .get(url)
            .header(COOKIE, format!("jwt_token={}", data.token))
            .send()
            .unwrap()
            .error_for_status()
            .expect("Failed to get user invite status")
            .json()
            .expect("Failed to parse user invite status response");

        res.num_weeks_available
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
        let link = format!("{BASE_INVITE_URL}/{code}");

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

    #[test]
    fn resolve_inviter_id() {
        // This is a real response header.
        let location_header = "https://af4a.adj.st/?adjust_t=tj1xyo&adjust_label=BDHTZTB5CWWKTVW2UCDTY27MBE&adjust_fallback=https%3A%2F%2Fwww.duolingo.com%2Freferred%3Fuser_invite%3DBDHTZTB5CWWKTVW2UCDTY27MBE%26inviter_id%3D925130045&adjust_redirect_macos=https%3A%2F%2Fwww.duolingo.com%2Freferred%3Fuser_invite%3DBDHTZTB5CWWKTVW2UCDTY27MBE%26inviter_id%3D925130045&adjust_deeplink=duolingo%3A%2F%2Fprofile%3Fuser_id%3D925130045";
        let inviter_id = String::from("925130045");

        assert_eq!(DuoApi::resolve_inviter_id(location_header), inviter_id)
    }

    #[test]
    #[should_panic]
    fn resolve_inviter_id_with_invalid_url() {
        DuoApi::resolve_inviter_id("...");
    }
}
