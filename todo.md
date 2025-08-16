- In backend/src/config.rs around lines 28-31, the closure map_err(|_| "…")
produces a &str/String which doesn’t implement std::error::Error and causes a
type mismatch; replace the map_err so it forwards the original VarError as a
boxed error (e.g. .map_err(|e| Box::<dyn std::error::Error>::from(e))?) and
after obtaining server_password check if it is empty and return an appropriate
boxed error (for example
Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput,
"SERVER_PASSWORD cannot be empty")))) if so; also add the missing import for
std::io (or std::io::Error/ErrorKind) at the top of the file.
