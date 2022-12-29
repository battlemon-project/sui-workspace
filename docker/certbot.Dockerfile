FROM certbot/certbot:v1.32.2
WORKDIR /app
COPY scripts/certbot-entrypoint.sh .
RUN chmod +x certbot-entrypoint.sh
ENTRYPOINT ["./certbot-entrypoint.sh"]