# Firebase Admin SDK for Rust
The Firebase Admin Rust SDK enables access to Firebase services from privileged environments. Designed to be scalable and reliable with zero-overhead for performance in mind.

# Currently supports
* GCP service accounts
* User and custom authentication management
* Firebase emulator integration and management
* Firebase OIDC token and session cookie verification using asynchronous public certificate cache

# Example for interacting with Firebase on GCP
```rust
use rs_firebase_admin_sdk::{
    auth::{FirebaseAuthService, UserIdentifiers},
    client::ApiHttpClient,
    App,
};

// Create live (not emulated) context for Firebase app
let live_app = App::live().await.unwrap();

// Create Firebase authentication admin client
let auth_admin = live_app.auth();

let user = auth_admin.get_user(
    // Build a filter for finding the user
    UserIdentifiers::builder()
        .with_email("me@email.com".into())
        .build()
)
.await
.expect("Error while fetching user")
.expect("User does not exist");

println!("User id: {}", user.uid);
```

# Example with an explicit project ID
Use `App::live_with_project_id` when your Firebase project differs from the GCP project in
`GOOGLE_CLOUD_PROJECT` — for example when identity management runs in a separate project from
Cloud Storage and other infrastructure services.

```rust
use rs_firebase_admin_sdk::{
    auth::{FirebaseAuthService, UserIdentifiers},
    client::ApiHttpClient,
    App,
};

// Supply the Firebase project ID directly instead of reading GOOGLE_CLOUD_PROJECT
let live_app = App::live_with_project_id("my-firebase-project-id").await.unwrap();

let auth_admin = live_app.auth();

let user = auth_admin.get_user(
    UserIdentifiers::builder()
        .with_email("me@email.com".into())
        .build()
)
.await
.expect("Error while fetching user")
.expect("User does not exist");

println!("User id: {}", user.uid);
```

# Custom token minting
Firebase custom tokens are RS256 JWTs signed by a service account via the IAM Credentials API.
The signing service account is resolved in this order:

1. **Explicit** — pass it to `auth_with_signer`:
   ```rust
   let auth_admin = live_app.auth_with_signer("my-sa@my-project.iam.gserviceaccount.com");
   let token = auth_admin.create_custom_token("user-uid").await?;
   ```

2. **Auto-discovered** — when running on GCE, Cloud Run, or GKE, the service account is fetched
   automatically from the instance metadata server. Just use `auth()`:
   ```rust
   let auth_admin = live_app.auth();
   let token = auth_admin.create_custom_token("user-uid").await?;
   ```

The service account must have the `iam.serviceAccounts.signJwt` permission
(`roles/iam.serviceAccountTokenCreator`).

Optional developer claims can be included:
```rust
use serde_json::json;

let claims = json!({ "premium": true, "role": "admin" });
let token = auth_admin.create_custom_token_with_claims("user-uid", claims).await?;
```

For more examples please see https://github.com/expl/rs-firebase-admin-sdk/tree/main/examples

# Running tests

Unit tests (no external dependencies, run from host):
```sh
cargo test -p rs-firebase-admin-sdk
```

Integration tests require the Firebase emulator. Start the Docker environment first, then:
```sh
docker compose -f docker/docker-compose.yaml up -d
docker exec firebase-admin-sdk cargo test -p rs-firebase-admin-sdk
```