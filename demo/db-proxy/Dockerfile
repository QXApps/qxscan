FROM alpine:3.20
RUN apk add --no-cache socat openssl bash
COPY pg-forward.sh /pg-forward.sh
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /pg-forward.sh /entrypoint.sh
EXPOSE 5433
CMD ["/entrypoint.sh"]
