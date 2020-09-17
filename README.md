# rust-http-proxy
A http proxy demo written with Rust/Tokio. 

Released in public domain as an example for Rust/Tokio.

## HTTPS Keygen
`keygen.sh` is used to generated a self-signed SSL certificate for HTTPS
proxy services. Simply runs it to generate a new cert. 

Testing it with `curl --insecure`, otherwise, the connection will be rejected
because self-signed certificates are not recognized by most clients.