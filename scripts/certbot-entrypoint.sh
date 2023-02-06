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
      --non-interactive \
      --standalone \
      --preferred-challenges=http \
      --domain $DOMAIN \
      --agree-tos \
      --email fedorovdanila@gmail.com \
      --rsa-key-size 4096 \
      --verbose \
      --keep-until-expiring &&
      export CERTBOT_RENEW=true
  fi

  sleep 12h &
  wait $!
done
