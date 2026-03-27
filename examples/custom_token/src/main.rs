/// Demonstrates Firebase custom token minting via the IAM Credentials API.
///
/// The signing service account is resolved in this order:
///   1. `FIREBASE_SIGNING_SA` env var (explicit override)
///   2. GCE metadata server auto-discovery (Cloud Run, GKE, GCE, Cloud Functions)
///
/// The signed token can be exchanged for a Firebase ID token by calling
/// `signInWithCustomToken` on the Firebase client SDK or REST API.
///
/// Required IAM permission on the signing service account:
///   iam.serviceAccounts.signJwt  (granted by roles/iam.serviceAccountTokenCreator)
///
/// Usage — explicit service account:
///   FIREBASE_PROJECT_ID=my-project \
///   FIREBASE_SIGNING_SA=my-sa@my-project.iam.gserviceaccount.com \
///   cargo run --example custom_token -- user-uid-123
///
/// Usage — auto-discover service account (Cloud Run / GCE / GKE):
///   FIREBASE_PROJECT_ID=my-project \
///   cargo run --example custom_token -- user-uid-123
use rs_firebase_admin_sdk::{App, auth::FirebaseAuthService};
use serde_json::json;

#[tokio::main]
async fn main() {
    let uid = std::env::args()
        .nth(1)
        .expect("Usage: custom_token <uid> [claims-json]");

    let custom_claims = std::env::args()
        .nth(2)
        .map(|s| serde_json::from_str(&s).expect("Second argument must be valid JSON"));

    let project_id = std::env::var("FIREBASE_PROJECT_ID").expect("FIREBASE_PROJECT_ID must be set");

    let app = App::live_with_project_id(&project_id)
        .await
        .expect("Failed to initialise Firebase app");

    let auth = match std::env::var("FIREBASE_SIGNING_SA") {
        Ok(sa_email) => {
            println!("Using explicit signing service account: {sa_email}");
            app.auth_with_signer(&sa_email)
        }
        Err(_) => {
            println!("No FIREBASE_SIGNING_SA set — will auto-discover from GCE metadata server");
            app.auth()
        }
    };

    let token = match custom_claims {
        Some(claims) => {
            println!("Minting custom token for uid={uid} with claims: {claims}");
            auth.create_custom_token_with_claims(&uid, claims)
                .await
                .expect("Failed to mint custom token")
        }
        None => {
            println!("Minting custom token for uid={uid}");
            auth.create_custom_token(&uid)
                .await
                .expect("Failed to mint custom token")
        }
    };

    println!("\nCustom token:\n{token}");
    println!("\nExchange this token via signInWithCustomToken on the Firebase client SDK.");

    // Example claims for reference — not used above unless passed as argument.
    let _example_claims = json!({
        "premium": true,
        "role": "admin"
    });
}
