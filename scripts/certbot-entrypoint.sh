#!/usr/bin/env sh

trap exit TERM
echo "Starting certbot"

while true; do
  if [ "$CERTBOT_RENEW" = true ]; then
    echo "Renewing certificate"
    certbot renew
  else
    echo "Getting new certificate"
    certbot certonly \
      --webroot \
      --webroot-path /tmp \
      --domain api.battlemon.com \
      --non-interactive \
      --agree-tos \
      --email fedorovdanila@gmail.com \
      --rsa-key-size 4096 \
      --verbose \
      --keep-until-expiring \
      --preferred-challenges=http &&
      export CERTBOT_RENEW=true
  fi

  sleep 12h &
  wait $!
done
