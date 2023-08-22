#!/bin/sh
openssl genrsa -out certificates/key.pem 2048
openssl req -new -x509 -key certificates/key.pem -out certificates/cert.pem -days 365