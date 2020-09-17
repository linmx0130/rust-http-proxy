#!env bash
openssl req -x509 -newkey rsa:4096 -keyout myKey.pem -out cert.pem -days 365 -nodes
openssl pkcs12 -export -out src/keyStore.p12 -inkey myKey.pem -in cert.pem
rm myKey.pem cert.pem