FROM nginx:1.21.6
WORKDIR /app
COPY scripts/nginx-entrypoint.sh .
RUN chmod +x nginx-entrypoint.sh
ENTRYPOINT ["./nginx-entrypoint.sh"]
