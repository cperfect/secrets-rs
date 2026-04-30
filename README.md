# Rust Secrets
`secrets-rs` is a library for safely retrieve and use secrets in rust applications. The primary purpose for this is configuring applications.

Safety means that the secret itself must be explicitly asked for by the application and all default ways of accessing it produce a masked value instead (see Value vs Masked Value).


## Core Concepts
### Secrets
A secret is any data that should not be generally accessible to the rest of the application or it
The initially supported data types will be:
* String (UTF-8)
* JSON
* raw bytes (users will need to further manipulate these)
A secret will be uniquely identified within the application by a urn of the form `urn:secret-rs:<source_id>:<name>` where the first two elements (the scheme and the NID) are case insensitive (as per the RFC8141) and the cases sensitivity of the latter two elements depend on the source type (these are the NSS, which can be sensitive per the RFC).
### Sources
Source are where secrets can be retrieved from. The initial supported source with Environment Variables. Future sources might include Cloud Services such as AWS Secrets Manager or Azure Key Vault. Sources are identified by Source Id which must be unique within the context of the application. The id could be logical or physical depending on the source type.
### Name
A secret is uniquely named within a source and a combination of name and source id and the combination of source id and name must be unique within the context of the application.
### Binding
Binding is the process of retieving a secrete from a source based on the source id and name and storing in a struct from which the value can be accessed. 
### Value vs Masked Value
The value is the actual secret data and must be explicitly asked for. The masked value is safe to print to logs etc and what will be returned by default e.g. by to string functions, default json mappings etc. The masked value will be the secret urn followed by the type and the size or length. Attempting to retreive the value before binding is an error. The maked value of an unbound secret will include the words "UNBOUND" instead of the length/size.


## Implementation
A struct Secret will be implemented with generic type argument for the value that can be used as a property in config objects.

Helper methods will be created to find all Secret Properties of a struct (recursively) and bind them, collecting and reporting any errors.

Additionally a custom attribute will be created with a macro to decorate properties of suitable types e.g

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
  #[secret("urn:secrets-rs:env:SUPER_SECRET_KEY")]
  pub key string
}
```
for use with serialisation and deserialisation, e.g. via serde. The intent is to replace deserialisation with bind and serialisation with the masked value.

Attempting to serialise or deserialise to a secret is an error.

## Out of Scope
* Writing Secrets to sources
* In-memory encryption






