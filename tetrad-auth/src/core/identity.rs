//@NOTE: Used to verify who issued an identity token and
//       which user it represents. `issuer` identifies the
//       OpenID provider that created and signed the token.
//       `subject` is identifies the unique ID for the user
//
//       https://openid.net/developers/how-connect-works/
pub struct AccountIdentity {
    issuer: String,
    subject: String,
}

impl AccountIdentity {
    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}
