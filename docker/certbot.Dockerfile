FROM certbot/certbot:arm64v8-v1.32.2
WORKDIR /app
COPY ./scripts/certbot-entrypoint.sh .
RUN chmod +x certbot-entrypoint.sh
ENTRYPOINT ["./certbot-entrypoint.sh"]