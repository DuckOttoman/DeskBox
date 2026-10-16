#!/bin/sh
set -eu
umask 077
/opt/deskbox-market/lego run --accept-tos --account-id deskbox-market --domains 47.85.175.203 --profile shortlived --http --path /var/lib/deskbox-market/acme --cert.timeout 30
src=/var/lib/deskbox-market/acme/certificates
out=/etc/deskbox-market/tls
if ! cmp -s "$src/47.85.175.203.crt" "$out/cert.pem"; then
 openssl x509 -in "$src/47.85.175.203.crt" -noout -checkend 3600
 install -m 640 -o root -g deskbox-market "$src/47.85.175.203.crt" "$out/cert.pem.new"
 install -m 640 -o root -g deskbox-market "$src/47.85.175.203.key" "$out/key.pem.new"
 mv "$out/cert.pem.new" "$out/cert.pem"
 mv "$out/key.pem.new" "$out/key.pem"
 systemctl restart deskbox-market.service
fi
