use aidoku::{
    alloc::{string::String, format},
    imports::defaults::{defaults_get, defaults_set, DefaultValue},
    imports::net::Request,
};

// Clé API publique Firebase (Identity Toolkit) utilisée par le frontend web de Comics Tracker.
// Il est normal et sécurisé que cette clé soit publique (elle est visible dans le code source du site web),
// car elle sert uniquement à identifier l'application cliente auprès de Firebase.
const FIREBASE_API_KEY: &str = "AIzaSyCDnQaQEvtoweE1rwsdb2gBIeDLtClWjbM";

/// Récupère un Bearer token Firebase, en utilisant le cache si disponible.
pub fn get_firebase_token() -> Option<String> {
    // Essayer le cache d'abord
    if let Some(cached) = defaults_get::<String>("ct_cached_token") {
        if !cached.is_empty() {
            return Some(cached);
        }
    }
    
    // Sinon, authentification Firebase
    let email: String = defaults_get("ct_email").unwrap_or_default();
    let password: String = defaults_get("ct_password").unwrap_or_default();
    
    if email.is_empty() || password.is_empty() {
        return None;
    }
    
    let auth_url = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key={}",
        FIREBASE_API_KEY
    );
    let payload = format!(
        "{{\"email\":\"{}\",\"password\":\"{}\",\"returnSecureToken\":true}}",
        email.replace("\"", "\\\""),
        password.replace("\"", "\\\"")
    );
    
    let mut auth_req = Request::post(&auth_url).ok()?;
    auth_req = auth_req.header("Content-Type", "application/json");
    auth_req = auth_req.body(payload.as_bytes());
    
    let bytes = auth_req.data().ok()?;
    let json: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let token = json.get("idToken")?.as_str()?;
    
    // Mettre en cache pour les prochains appels
    defaults_set("ct_cached_token", DefaultValue::String(String::from(token)));
    
    Some(String::from(token))
}

/// Invalide le cache du token (à appeler si on reçoit un 401).
pub fn invalidate_token_cache() {
    defaults_set("ct_cached_token", DefaultValue::String(String::new()));
}
